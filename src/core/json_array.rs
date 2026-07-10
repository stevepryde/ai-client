use std::collections::VecDeque;

use futures::{stream, StreamExt};
use serde::de::DeserializeOwned;

use crate::{
    error::AiProvider,
    stream::{AiStream, AiStreamError, AiStreamErrorKind},
};

pub(crate) fn values<T>(
    bytes: AiStream<Vec<u8>>,
    provider: AiProvider,
    operation: &'static str,
) -> AiStream<T>
where
    T: DeserializeOwned + Send + 'static,
{
    struct State<T> {
        bytes: AiStream<Vec<u8>>,
        decoder: Decoder,
        pending: VecDeque<Result<T, AiStreamError>>,
        ended: bool,
    }

    let state = State {
        bytes,
        decoder: Decoder::new(provider, operation),
        pending: VecDeque::new(),
        ended: false,
    };
    AiStream::new(stream::unfold(state, |mut state| async move {
        loop {
            if let Some(item) = state.pending.pop_front() {
                return Some((item, state));
            }
            if state.ended || state.decoder.failed || state.decoder.phase == Phase::End {
                return None;
            }
            match state.bytes.next().await {
                Some(Ok(chunk)) => state.pending.extend(state.decoder.push::<T>(&chunk)),
                Some(Err(error)) => {
                    state.ended = true;
                    return Some((Err(error), state));
                }
                None => {
                    state.pending.extend(state.decoder.finish::<T>());
                    state.ended = true;
                }
            }
        }
    }))
}

#[derive(Clone, Copy, PartialEq, Eq)]
enum Phase {
    Start,
    ValueOrEnd,
    Delimiter,
    End,
}

struct Decoder {
    provider: AiProvider,
    operation: &'static str,
    buffer: Vec<u8>,
    phase: Phase,
    failed: bool,
}

impl Decoder {
    fn new(provider: AiProvider, operation: &'static str) -> Self {
        Self {
            provider,
            operation,
            buffer: Vec::new(),
            phase: Phase::Start,
            failed: false,
        }
    }

    fn push<T>(&mut self, bytes: &[u8]) -> Vec<Result<T, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        self.buffer.extend_from_slice(bytes);
        self.process(false)
    }

    fn finish<T>(&mut self) -> Vec<Result<T, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        self.process(true)
    }

    fn process<T>(&mut self, eof: bool) -> Vec<Result<T, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        let mut output = Vec::new();
        loop {
            self.discard_whitespace();
            match self.phase {
                Phase::Start => match self.buffer.first() {
                    Some(b'[') => {
                        self.buffer.drain(..1);
                        self.phase = Phase::ValueOrEnd;
                    }
                    Some(_) => {
                        output.push(Err(self.fail(AiStreamErrorKind::MalformedJson(
                            json_error(b"expected array"),
                        ))));
                        break;
                    }
                    None if eof => {
                        output.push(Err(self.fail(AiStreamErrorKind::UnexpectedEof)));
                        break;
                    }
                    None => break,
                },
                Phase::ValueOrEnd => match self.buffer.first() {
                    Some(b']') => {
                        self.buffer.drain(..1);
                        self.phase = Phase::End;
                    }
                    Some(_) => {
                        let mut values =
                            serde_json::Deserializer::from_slice(&self.buffer).into_iter::<T>();
                        match values.next() {
                            Some(Ok(value)) => {
                                let consumed = values.byte_offset();
                                self.buffer.drain(..consumed);
                                self.phase = Phase::Delimiter;
                                output.push(Ok(value));
                            }
                            Some(Err(error)) if error.is_eof() && !eof => break,
                            Some(Err(error)) if error.is_eof() => {
                                output.push(Err(self.fail(AiStreamErrorKind::UnexpectedEof)));
                                break;
                            }
                            Some(Err(error)) => {
                                let kind = match std::str::from_utf8(&self.buffer) {
                                    Err(_) => AiStreamErrorKind::InvalidUtf8,
                                    Ok(_) => AiStreamErrorKind::MalformedJson(error.into()),
                                };
                                output.push(Err(self.fail(kind)));
                                break;
                            }
                            None => break,
                        }
                    }
                    None if eof => {
                        output.push(Err(self.fail(AiStreamErrorKind::UnexpectedEof)));
                        break;
                    }
                    None => break,
                },
                Phase::Delimiter => match self.buffer.first() {
                    Some(b',') => {
                        self.buffer.drain(..1);
                        self.phase = Phase::ValueOrEnd;
                    }
                    Some(b']') => {
                        self.buffer.drain(..1);
                        self.phase = Phase::End;
                    }
                    Some(_) => {
                        output.push(Err(self.fail(AiStreamErrorKind::MalformedJson(
                            json_error(b"expected delimiter"),
                        ))));
                        break;
                    }
                    None if eof => {
                        output.push(Err(self.fail(AiStreamErrorKind::UnexpectedEof)));
                        break;
                    }
                    None => break,
                },
                Phase::End => {
                    if !self.buffer.is_empty() {
                        output.push(Err(self.fail(AiStreamErrorKind::MalformedJson(
                            json_error(b"trailing data"),
                        ))));
                    }
                    break;
                }
            }
        }
        output
    }

    fn discard_whitespace(&mut self) {
        let count = self
            .buffer
            .iter()
            .take_while(|byte| byte.is_ascii_whitespace())
            .count();
        self.buffer.drain(..count);
    }

    fn fail(&mut self, kind: AiStreamErrorKind) -> AiStreamError {
        self.failed = true;
        AiStreamError::new(self.provider, self.operation, kind)
    }
}

fn json_error(input: &[u8]) -> crate::error::JsonDecodeError {
    serde_json::from_slice::<serde_json::Value>(input)
        .expect_err("test input must be invalid JSON")
        .into()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::error::TransportErrorKind;
    use futures::{stream, StreamExt};

    #[derive(Debug, serde::Deserialize, PartialEq, Eq)]
    struct Item {
        value: u32,
    }

    fn byte_stream(chunks: Vec<Vec<u8>>) -> AiStream<Vec<u8>> {
        AiStream::new(stream::iter(chunks.into_iter().map(Ok)))
    }

    async fn collect(chunks: Vec<Vec<u8>>) -> Vec<Result<Item, AiStreamError>> {
        values(byte_stream(chunks), AiProvider::Gemini, "test.json_array")
            .collect()
            .await
    }

    #[tokio::test]
    async fn handles_empty_multiple_and_every_byte_chunk_split() {
        assert!(collect(vec![b"[]".to_vec()]).await.is_empty());

        let body = br#"[{"value":1},{"value":2}]"#;
        let chunks = body.iter().map(|byte| vec![*byte]).collect();
        let items = collect(chunks)
            .await
            .into_iter()
            .collect::<Result<Vec<_>, _>>()
            .unwrap();
        assert_eq!(items, vec![Item { value: 1 }, Item { value: 2 }]);
    }

    #[tokio::test]
    async fn malformed_invalid_utf8_eof_and_interruption_are_redacted() {
        let malformed = collect(vec![b"[{\"private-secret\":}]".to_vec()])
            .await
            .remove(0)
            .unwrap_err();
        assert!(matches!(
            malformed.kind(),
            AiStreamErrorKind::MalformedJson(_)
        ));
        assert!(!format!("{malformed:?}").contains("private-secret"));

        let invalid = collect(vec![vec![b'[', 0xff, b']']])
            .await
            .remove(0)
            .unwrap_err();
        assert!(matches!(invalid.kind(), AiStreamErrorKind::InvalidUtf8));

        let truncated = collect(vec![br#"[{"value":1}"#.to_vec()]).await;
        assert_eq!(truncated[0].as_ref().unwrap(), &Item { value: 1 });
        assert!(matches!(
            truncated[1].as_ref().unwrap_err().kind(),
            AiStreamErrorKind::UnexpectedEof
        ));

        let interruption = AiStreamError::new(
            AiProvider::Gemini,
            "test.json_array",
            AiStreamErrorKind::Transport(TransportErrorKind::Body),
        );
        let bytes = AiStream::new(stream::iter(vec![
            Ok(b"[{\"private-stream-payload\":".to_vec()),
            Err(interruption.clone()),
        ]));
        let error = values::<Item>(bytes, AiProvider::Gemini, "test.json_array")
            .next()
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(error, interruption);
        assert!(!format!("{error:?}").contains("private-stream-payload"));
    }
}
