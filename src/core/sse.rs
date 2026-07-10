use std::{collections::VecDeque, marker::PhantomData, time::Duration};

use futures::{stream, StreamExt};
use serde::de::DeserializeOwned;

use crate::{
    error::AiProvider,
    stream::{AiStream, AiStreamError, AiStreamErrorKind, SseEventMetadata, SseJsonEvent},
};

pub(crate) fn json_events<T>(
    bytes: AiStream<Vec<u8>>,
    provider: AiProvider,
    operation: &'static str,
) -> AiStream<SseJsonEvent<T>>
where
    T: DeserializeOwned + Send + 'static,
{
    struct State<T> {
        bytes: AiStream<Vec<u8>>,
        decoder: Decoder,
        pending: VecDeque<Result<SseJsonEvent<T>, AiStreamError>>,
        ended: bool,
        marker: PhantomData<T>,
    }

    let state = State {
        bytes,
        decoder: Decoder::new(provider, operation),
        pending: VecDeque::new(),
        ended: false,
        marker: PhantomData,
    };
    AiStream::new(stream::unfold(state, |mut state| async move {
        loop {
            if let Some(item) = state.pending.pop_front() {
                return Some((item, state));
            }
            if state.ended || state.decoder.done {
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

struct Decoder {
    provider: AiProvider,
    operation: &'static str,
    buffer: Vec<u8>,
    event: EventBuilder,
    done: bool,
    failed: bool,
}

impl Decoder {
    fn new(provider: AiProvider, operation: &'static str) -> Self {
        Self {
            provider,
            operation,
            buffer: Vec::new(),
            event: EventBuilder::default(),
            done: false,
            failed: false,
        }
    }

    fn push<T>(&mut self, bytes: &[u8]) -> Vec<Result<SseJsonEvent<T>, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        self.buffer.extend_from_slice(bytes);
        self.process::<T>(false)
    }

    fn finish<T>(&mut self) -> Vec<Result<SseJsonEvent<T>, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        self.process::<T>(true)
    }

    fn process<T>(&mut self, eof: bool) -> Vec<Result<SseJsonEvent<T>, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        let mut output = Vec::new();
        while !self.failed && !self.done {
            let Some(line) = self.take_line(eof) else {
                break;
            };
            if line.is_empty() {
                if let Some(event) = self.finish_event::<T>() {
                    output.push(event);
                }
                continue;
            }
            let line = match std::str::from_utf8(&line) {
                Ok(line) => line,
                Err(_) => {
                    self.failed = true;
                    output.push(Err(self.error(AiStreamErrorKind::InvalidUtf8)));
                    break;
                }
            };
            self.event.push_line(line);
        }
        if eof && !self.failed && !self.done {
            if let Some(event) = self.finish_event::<T>() {
                output.push(event);
            }
        }
        output
    }

    fn take_line(&mut self, eof: bool) -> Option<Vec<u8>> {
        let position = self
            .buffer
            .iter()
            .position(|byte| matches!(*byte, b'\n' | b'\r'));
        match position {
            Some(position) => {
                if self.buffer[position] == b'\r' && position + 1 == self.buffer.len() && !eof {
                    return None;
                }
                let line = self.buffer[..position].to_vec();
                let ending = if self.buffer[position] == b'\r'
                    && self.buffer.get(position + 1) == Some(&b'\n')
                {
                    2
                } else {
                    1
                };
                self.buffer.drain(..position + ending);
                Some(line)
            }
            None if eof && !self.buffer.is_empty() => Some(std::mem::take(&mut self.buffer)),
            None => None,
        }
    }

    fn finish_event<T>(&mut self) -> Option<Result<SseJsonEvent<T>, AiStreamError>>
    where
        T: DeserializeOwned,
    {
        let event = std::mem::take(&mut self.event);
        if event.data.is_empty() {
            return None;
        }
        let data = event.data.join("\n");
        if data.trim() == "[DONE]" {
            self.done = true;
            return None;
        }
        let raw = match serde_json::from_str::<serde_json::Value>(&data) {
            Ok(raw) => raw,
            Err(error) => {
                self.failed = true;
                return Some(Err(
                    self.error(AiStreamErrorKind::MalformedJson(error.into()))
                ));
            }
        };
        let typed = match serde_json::from_value::<T>(raw.clone()) {
            Ok(typed) => typed,
            Err(error) => {
                self.failed = true;
                return Some(Err(
                    self.error(AiStreamErrorKind::MalformedJson(error.into()))
                ));
            }
        };
        Some(Ok(SseJsonEvent::new(
            typed,
            raw,
            SseEventMetadata::new(event.name, event.id, event.retry),
        )))
    }

    fn error(&self, kind: AiStreamErrorKind) -> AiStreamError {
        AiStreamError::new(self.provider, self.operation, kind)
    }
}

#[derive(Default)]
struct EventBuilder {
    data: Vec<String>,
    name: Option<String>,
    id: Option<String>,
    retry: Option<Duration>,
}

impl EventBuilder {
    fn push_line(&mut self, line: &str) {
        if line.starts_with(':') {
            return;
        }
        let (field, value) = line.split_once(':').map_or((line, ""), |(field, value)| {
            (field, value.strip_prefix(' ').unwrap_or(value))
        });
        match field {
            "data" => self.data.push(value.to_string()),
            "event" => self.name = Some(value.to_string()),
            "id" if !value.contains('\0') => self.id = Some(value.to_string()),
            "retry" => {
                if let Ok(milliseconds) = value.parse::<u64>() {
                    self.retry = Some(Duration::from_millis(milliseconds));
                }
            }
            _ => {}
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::{
        error::TransportErrorKind,
        stream::{AiStreamErrorKind, SseJsonEvent},
    };
    use futures::{stream, StreamExt};

    fn byte_stream(chunks: Vec<Vec<u8>>) -> AiStream<Vec<u8>> {
        AiStream::new(stream::iter(chunks.into_iter().map(Ok)))
    }

    async fn collect<T>(chunks: Vec<Vec<u8>>) -> Vec<Result<SseJsonEvent<T>, AiStreamError>>
    where
        T: DeserializeOwned + Send + 'static,
    {
        json_events(byte_stream(chunks), AiProvider::OpenAI, "test.sse")
            .collect()
            .await
    }

    #[derive(Debug, serde::Deserialize, PartialEq)]
    #[serde(tag = "type", rename_all = "snake_case")]
    enum Tagged {
        Known {
            text: String,
        },
        #[serde(other)]
        Unknown,
    }

    #[tokio::test]
    async fn every_chunk_split_preserves_utf8_and_unknown_raw_json() {
        let body = r#"data: {"type":"future_event","text":"café","extra":42}

"#
        .as_bytes();
        for split in 0..=body.len() {
            let events =
                collect::<Tagged>(vec![body[..split].to_vec(), body[split..].to_vec()]).await;
            assert_eq!(events.len(), 1, "split {split}");
            let event = events.into_iter().next().unwrap().unwrap();
            assert_eq!(event.data(), &Tagged::Unknown);
            assert_eq!(event.raw()["text"], "café");
            assert_eq!(event.raw()["extra"], 42);
        }
    }

    #[tokio::test]
    async fn handles_crlf_comments_multiline_metadata_blank_done_and_eof() {
        let body = b": comment\r\n\r\nevent: response.delta\r\nid: evt_7\r\nretry: 1500\r\ndata: {\"value\":\r\ndata: 1}\r\n\r\n\r\ndata: [DONE]\r\n\r\ndata: {\"ignored\":true}\r\n\r\n";
        let events = collect::<serde_json::Value>(vec![body.to_vec()]).await;
        assert_eq!(events.len(), 1);
        let event = events.into_iter().next().unwrap().unwrap();
        assert_eq!(event.data()["value"], 1);
        assert_eq!(event.metadata().event(), Some("response.delta"));
        assert_eq!(event.metadata().id(), Some("evt_7"));
        assert_eq!(event.metadata().retry(), Some(Duration::from_millis(1500)));

        let eof = collect::<serde_json::Value>(vec![b"data: {\"final\":true}".to_vec()]).await;
        assert_eq!(eof.len(), 1);
        assert_eq!(eof[0].as_ref().unwrap().data()["final"], true);

        let multiple = collect::<serde_json::Value>(vec![
            b"data: {\"sequence\":1}\n\ndata: {\"sequence\":2}\n\n".to_vec(),
        ])
        .await;
        assert_eq!(multiple.len(), 2);
        assert_eq!(multiple[0].as_ref().unwrap().data()["sequence"], 1);
        assert_eq!(multiple[1].as_ref().unwrap().data()["sequence"], 2);
    }

    #[tokio::test]
    async fn malformed_json_invalid_utf8_and_interruption_are_redacted() {
        let malformed =
            collect::<serde_json::Value>(vec![b"data: {\"private-secret\":\n\n".to_vec()])
                .await
                .remove(0)
                .unwrap_err();
        assert!(matches!(
            malformed.kind(),
            AiStreamErrorKind::MalformedJson(_)
        ));
        assert!(!format!("{malformed:?}").contains("private-secret"));
        assert!(!malformed.to_string().contains("private-secret"));

        let invalid = collect::<serde_json::Value>(vec![
            b"data: {\"text\":\"".to_vec(),
            vec![0xff, b'"', b'}', b'\n', b'\n'],
        ])
        .await
        .remove(0)
        .unwrap_err();
        assert!(matches!(invalid.kind(), AiStreamErrorKind::InvalidUtf8));

        let interruption = AiStreamError::new(
            AiProvider::OpenAI,
            "test.sse",
            AiStreamErrorKind::Transport(TransportErrorKind::Body),
        );
        let bytes = AiStream::new(stream::iter(vec![
            Ok(b"data: private-stream-payload".to_vec()),
            Err(interruption.clone()),
        ]));
        let error = json_events::<serde_json::Value>(bytes, AiProvider::OpenAI, "test.sse")
            .next()
            .await
            .unwrap()
            .unwrap_err();
        assert_eq!(error, interruption);
        assert!(!format!("{error:?}").contains("private-stream-payload"));
    }
}
