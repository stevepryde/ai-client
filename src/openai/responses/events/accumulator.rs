use std::collections::BTreeMap;

use super::super::output::OpenAIResponsesCreateResponse;
use super::OpenAIResponsesStreamEvent;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ResponseStreamProgress {
    Started,
    Advanced,
    Completed,
}

#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum ResponseStreamAccumulatorError {
    #[error("stream event sequence is out of order: expected {expected}, got {actual}")]
    OutOfOrder { expected: u64, actual: u64 },
    #[error("received an event after a terminal response event")]
    EventAfterTerminal,
    #[error("received a terminal response before any lifecycle response")]
    TerminalBeforeLifecycle,
    #[error("stream event is missing a sequence_number")]
    MissingSequenceNumber,
    #[error("provider stream error {code:?}: {message}")]
    Provider {
        code: Option<String>,
        message: String,
        param: Option<String>,
    },
}

#[derive(Debug, Default)]
pub struct ResponseStreamAccumulator {
    last_sequence: Option<u64>,
    latest: Option<OpenAIResponsesCreateResponse>,
    text: BTreeMap<(u64, u64, String), String>,
    terminal: bool,
    finalized: bool,
}

impl ResponseStreamAccumulator {
    pub fn new() -> Self {
        Self::default()
    }

    pub fn push(
        &mut self,
        event: OpenAIResponsesStreamEvent,
    ) -> Result<ResponseStreamProgress, ResponseStreamAccumulatorError> {
        if self.terminal {
            return Err(ResponseStreamAccumulatorError::EventAfterTerminal);
        }
        let actual = event
            .sequence_number()
            .ok_or(ResponseStreamAccumulatorError::MissingSequenceNumber)?;
        if let Some(previous) = self.last_sequence {
            let expected = previous.saturating_add(1);
            if actual != expected {
                return Err(ResponseStreamAccumulatorError::OutOfOrder { expected, actual });
            }
        }
        self.last_sequence = Some(actual);

        match event {
            OpenAIResponsesStreamEvent::Error(error) => {
                self.terminal = true;
                Err(ResponseStreamAccumulatorError::Provider {
                    code: error.code,
                    message: error.message,
                    param: error.param,
                })
            }
            OpenAIResponsesStreamEvent::ResponseCreated(event)
            | OpenAIResponsesStreamEvent::ResponseQueued(event)
            | OpenAIResponsesStreamEvent::ResponseInProgress(event) => {
                let started = self.latest.is_none();
                self.latest = Some(event.response);
                Ok(if started {
                    ResponseStreamProgress::Started
                } else {
                    ResponseStreamProgress::Advanced
                })
            }
            OpenAIResponsesStreamEvent::ResponseDone(event)
            | OpenAIResponsesStreamEvent::ResponseFailed(event)
            | OpenAIResponsesStreamEvent::ResponseIncomplete(event) => {
                if self.latest.is_none() {
                    return Err(ResponseStreamAccumulatorError::TerminalBeforeLifecycle);
                }
                self.latest = Some(event.response);
                self.terminal = true;
                self.finalized = true;
                Ok(ResponseStreamProgress::Completed)
            }
            OpenAIResponsesStreamEvent::OutputTextDelta(event) => {
                self.text
                    .entry((event.output_index, event.content_index, event.item_id))
                    .or_default()
                    .push_str(&event.delta);
                Ok(ResponseStreamProgress::Advanced)
            }
            _ => Ok(ResponseStreamProgress::Advanced),
        }
    }

    pub fn latest_response(&self) -> Option<&OpenAIResponsesCreateResponse> {
        self.latest.as_ref()
    }

    pub fn final_response(&self) -> Option<&OpenAIResponsesCreateResponse> {
        self.finalized.then_some(self.latest.as_ref()).flatten()
    }

    pub fn accumulated_output_text(&self) -> String {
        self.text.values().cloned().collect()
    }

    pub fn last_sequence_number(&self) -> Option<u64> {
        self.last_sequence
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn response(status: &str, output_text: &str) -> serde_json::Value {
        serde_json::json!({
            "metadata": {}, "top_logprobs": null, "temperature": 1.0, "top_p": 1.0,
            "user": null, "safety_identifier": null, "prompt_cache_key": null,
            "service_tier": "default", "prompt_cache_retention": null,
            "previous_response_id": null, "id": "resp_1", "object": "response",
            "status": status, "created_at": 1, "completed_at": null,
            "background": false, "max_tool_calls": null, "text": null,
            "error": null, "incomplete_details": null, "output": [],
            "reasoning": {}, "instructions": null, "output_text": output_text,
            "usage": null, "prompt_cache_options": null, "moderation": null,
            "parallel_tool_calls": true, "conversation": null, "max_output_tokens": null,
            "model": "gpt-5.4", "tools": [], "tool_choice": "auto",
            "prompt": null, "truncation": "disabled", "store": true
        })
    }

    #[test]
    fn created_delta_completed_reconstructs_and_yields_final_response() {
        let events = [
            serde_json::json!({"type":"response.created","response":response("in_progress", ""),"sequence_number":0}),
            serde_json::json!({"type":"response.output_text.delta","item_id":"msg_1","output_index":0,"content_index":0,"delta":"hello","logprobs":[],"sequence_number":1}),
            serde_json::json!({"type":"response.completed","response":response("completed", "hello"),"sequence_number":2}),
        ];
        let mut accumulator = ResponseStreamAccumulator::new();
        for value in events {
            accumulator
                .push(serde_json::from_value(value).unwrap())
                .unwrap();
        }
        assert_eq!(accumulator.accumulated_output_text(), "hello");
        assert_eq!(accumulator.final_response().unwrap().output_text, "hello");
    }

    #[test]
    fn rejects_out_of_order_and_terminal_before_lifecycle() {
        let mut accumulator = ResponseStreamAccumulator::new();
        accumulator
            .push(
                serde_json::from_value(serde_json::json!({
                    "type":"response.output_text.delta","item_id":"msg_1","output_index":0,
                    "content_index":0,"delta":"x","logprobs":[],"sequence_number":4
                }))
                .unwrap(),
            )
            .unwrap();
        let error = accumulator
            .push(
                serde_json::from_value(serde_json::json!({
                    "type":"response.output_text.done","item_id":"msg_1","output_index":0,
                    "content_index":0,"text":"x","logprobs":[],"sequence_number":6
                }))
                .unwrap(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ResponseStreamAccumulatorError::OutOfOrder { .. }
        ));

        let mut accumulator = ResponseStreamAccumulator::new();
        let error = accumulator
            .push(
                serde_json::from_value(serde_json::json!({
                    "type":"response.completed","response":response("completed", "x"),
                    "sequence_number":0
                }))
                .unwrap(),
            )
            .unwrap_err();
        assert_eq!(
            error,
            ResponseStreamAccumulatorError::TerminalBeforeLifecycle
        );
    }

    #[test]
    fn orders_text_by_output_position_not_item_id() {
        let mut accumulator = ResponseStreamAccumulator::new();
        for value in [
            serde_json::json!({"type":"response.created","response":response("in_progress", ""),"sequence_number":0}),
            serde_json::json!({"type":"response.output_text.delta","item_id":"z_item","output_index":0,"content_index":0,"delta":"first","logprobs":[],"sequence_number":1}),
            serde_json::json!({"type":"response.output_text.delta","item_id":"a_item","output_index":1,"content_index":0,"delta":"second","logprobs":[],"sequence_number":2}),
        ] {
            accumulator
                .push(serde_json::from_value(value).unwrap())
                .unwrap();
        }
        assert_eq!(accumulator.accumulated_output_text(), "firstsecond");
    }

    #[test]
    fn provider_error_does_not_expose_stale_response_as_final() {
        let mut accumulator = ResponseStreamAccumulator::new();
        accumulator
            .push(
                serde_json::from_value(serde_json::json!({
                    "type":"response.created","response":response("in_progress", ""),
                    "sequence_number":0
                }))
                .unwrap(),
            )
            .unwrap();
        let error = accumulator
            .push(
                serde_json::from_value(serde_json::json!({
                    "type":"error","code":"bad","message":"failed","param":null,
                    "sequence_number":1
                }))
                .unwrap(),
            )
            .unwrap_err();
        assert!(matches!(
            error,
            ResponseStreamAccumulatorError::Provider { .. }
        ));
        assert!(accumulator.latest_response().is_some());
        assert!(accumulator.final_response().is_none());
    }
}
