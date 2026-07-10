//! Typed server-sent event protocol for the Responses API.

mod accumulator;
mod common;
mod lifecycle;
mod media;
mod reasoning;
mod text;
mod tools;

pub use accumulator::*;
pub use common::*;
pub use lifecycle::*;
pub use media::*;
pub use reasoning::*;
pub use text::*;
pub use tools::*;

use super::tagged::lossless_tagged_enum;

lossless_tagged_enum! {
    #[derive(Debug, Clone)]
    pub enum OpenAIResponsesStreamEvent {
        AudioDelta(OpenAIAudioDeltaEvent) => "response.audio.delta",
        AudioDone(OpenAIAudioDoneEvent) => "response.audio.done",
        AudioTranscriptDelta(OpenAIAudioTranscriptDeltaEvent) => "response.audio.transcript.delta",
        AudioTranscriptDone(OpenAIAudioTranscriptDoneEvent) => "response.audio.transcript.done",
        CodeInterpreterCallCodeDelta(OpenAICodeInterpreterCodeDeltaEvent) => "response.code_interpreter_call_code.delta",
        CodeInterpreterCallCodeDone(OpenAICodeInterpreterCodeDoneEvent) => "response.code_interpreter_call_code.done",
        CodeInterpreterCallCompleted(OpenAIToolCallStatusEvent) => "response.code_interpreter_call.completed",
        CodeInterpreterCallInProgress(OpenAIToolCallStatusEvent) => "response.code_interpreter_call.in_progress",
        CodeInterpreterCallInterpreting(OpenAIToolCallStatusEvent) => "response.code_interpreter_call.interpreting",
        ResponseDone(OpenAIResponseEvent) => "response.completed",
        ContentPartAdded(OpenAIResponseContentPartEvent) => "response.content_part.added",
        ContentPartDone(OpenAIResponseContentPartEvent) => "response.content_part.done",
        ResponseCreated(OpenAIResponseEvent) => "response.created",
        Error(OpenAIStreamError) => "error",
        FileSearchCallCompleted(OpenAIToolCallStatusEvent) => "response.file_search_call.completed",
        FileSearchCallInProgress(OpenAIToolCallStatusEvent) => "response.file_search_call.in_progress",
        FileSearchCallSearching(OpenAIToolCallStatusEvent) => "response.file_search_call.searching",
        FunctionCallArgumentsDelta(OpenAIItemDeltaEvent) => "response.function_call_arguments.delta",
        FunctionCallArgumentsDone(OpenAIFunctionArgumentsDoneEvent) => "response.function_call_arguments.done",
        ResponseInProgress(OpenAIResponseEvent) => "response.in_progress",
        ResponseFailed(OpenAIResponseEvent) => "response.failed",
        ResponseIncomplete(OpenAIResponseEvent) => "response.incomplete",
        OutputItemAdded(OpenAIResponseOutputItemEvent) => "response.output_item.added",
        OutputItemDone(OpenAIResponseOutputItemEvent) => "response.output_item.done",
        ReasoningSummaryPartAdded(OpenAIReasoningSummaryPartEvent) => "response.reasoning_summary_part.added",
        ReasoningSummaryPartDone(OpenAIReasoningSummaryPartDoneEvent) => "response.reasoning_summary_part.done",
        ReasoningSummaryTextDelta(OpenAIReasoningSummaryDeltaEvent) => "response.reasoning_summary_text.delta",
        ReasoningSummaryTextDone(OpenAIReasoningSummaryDoneEvent) => "response.reasoning_summary_text.done",
        ReasoningTextDelta(OpenAIResponseTextDeltaEvent) => "response.reasoning_text.delta",
        ReasoningTextDone(OpenAIResponseTextDoneEvent) => "response.reasoning_text.done",
        RefusalDelta(OpenAIResponseTextDeltaEvent) => "response.refusal.delta",
        RefusalDone(OpenAIResponseRefusalDoneEvent) => "response.refusal.done",
        OutputTextDelta(OpenAIResponseOutputTextDelta) => "response.output_text.delta",
        OutputTextDone(OpenAIResponseOutputTextDone) => "response.output_text.done",
        WebSearchCallCompleted(OpenAIToolCallStatusEvent) => "response.web_search_call.completed",
        WebSearchCallInProgress(OpenAIToolCallStatusEvent) => "response.web_search_call.in_progress",
        WebSearchCallSearching(OpenAIToolCallStatusEvent) => "response.web_search_call.searching",
        ImageGenerationCallCompleted(OpenAIToolCallStatusEvent) => "response.image_generation_call.completed",
        ImageGenerationCallGenerating(OpenAIToolCallStatusEvent) => "response.image_generation_call.generating",
        ImageGenerationCallInProgress(OpenAIToolCallStatusEvent) => "response.image_generation_call.in_progress",
        ImageGenerationPartialImage(OpenAIImageGenerationPartialEvent) => "response.image_generation_call.partial_image",
        McpCallArgumentsDelta(OpenAIItemDeltaEvent) => "response.mcp_call_arguments.delta",
        McpCallArgumentsDone(OpenAIMcpArgumentsDoneEvent) => "response.mcp_call_arguments.done",
        McpCallCompleted(OpenAIToolCallStatusEvent) => "response.mcp_call.completed",
        McpCallFailed(OpenAIToolCallStatusEvent) => "response.mcp_call.failed",
        McpCallInProgress(OpenAIToolCallStatusEvent) => "response.mcp_call.in_progress",
        McpListToolsCompleted(OpenAIToolCallStatusEvent) => "response.mcp_list_tools.completed",
        McpListToolsFailed(OpenAIToolCallStatusEvent) => "response.mcp_list_tools.failed",
        McpListToolsInProgress(OpenAIToolCallStatusEvent) => "response.mcp_list_tools.in_progress",
        OutputTextAnnotationAdded(OpenAIAnnotationAddedEvent) => "response.output_text.annotation.added",
        ResponseQueued(OpenAIResponseEvent) => "response.queued",
        CustomToolCallInputDelta(OpenAIItemDeltaEvent) => "response.custom_tool_call_input.delta",
        CustomToolCallInputDone(OpenAICustomToolInputDoneEvent) => "response.custom_tool_call_input.done",
        @unknown
    }
}

impl OpenAIResponsesStreamEvent {
    pub fn sequence_number(&self) -> Option<u64> {
        match self {
            Self::AudioDelta(event) => Some(event.sequence_number),
            Self::AudioDone(event) => Some(event.sequence_number),
            Self::AudioTranscriptDelta(event) => Some(event.sequence_number),
            Self::AudioTranscriptDone(event) => Some(event.sequence_number),
            Self::CodeInterpreterCallCodeDelta(event) => Some(event.sequence_number),
            Self::CodeInterpreterCallCodeDone(event) => Some(event.sequence_number),
            Self::CodeInterpreterCallCompleted(event)
            | Self::CodeInterpreterCallInProgress(event)
            | Self::CodeInterpreterCallInterpreting(event)
            | Self::FileSearchCallCompleted(event)
            | Self::FileSearchCallInProgress(event)
            | Self::FileSearchCallSearching(event)
            | Self::WebSearchCallCompleted(event)
            | Self::WebSearchCallInProgress(event)
            | Self::WebSearchCallSearching(event)
            | Self::ImageGenerationCallCompleted(event)
            | Self::ImageGenerationCallGenerating(event)
            | Self::ImageGenerationCallInProgress(event)
            | Self::McpCallCompleted(event)
            | Self::McpCallFailed(event)
            | Self::McpCallInProgress(event)
            | Self::McpListToolsCompleted(event)
            | Self::McpListToolsFailed(event)
            | Self::McpListToolsInProgress(event) => Some(event.sequence_number),
            Self::ResponseDone(event)
            | Self::ResponseCreated(event)
            | Self::ResponseInProgress(event)
            | Self::ResponseFailed(event)
            | Self::ResponseIncomplete(event)
            | Self::ResponseQueued(event) => Some(event.sequence_number),
            Self::ContentPartAdded(event) | Self::ContentPartDone(event) => {
                Some(event.sequence_number)
            }
            Self::Error(event) => Some(event.sequence_number),
            Self::FunctionCallArgumentsDelta(event)
            | Self::McpCallArgumentsDelta(event)
            | Self::CustomToolCallInputDelta(event) => Some(event.sequence_number),
            Self::FunctionCallArgumentsDone(event) => Some(event.sequence_number),
            Self::OutputItemAdded(event) | Self::OutputItemDone(event) => {
                Some(event.sequence_number)
            }
            Self::ReasoningSummaryPartAdded(event) => Some(event.sequence_number),
            Self::ReasoningSummaryPartDone(event) => Some(event.sequence_number),
            Self::ReasoningSummaryTextDelta(event) => Some(event.sequence_number),
            Self::ReasoningSummaryTextDone(event) => Some(event.sequence_number),
            Self::ReasoningTextDelta(event) | Self::RefusalDelta(event) => {
                Some(event.sequence_number)
            }
            Self::ReasoningTextDone(event) => Some(event.sequence_number),
            Self::RefusalDone(event) => Some(event.sequence_number),
            Self::OutputTextDelta(event) => Some(event.sequence_number),
            Self::OutputTextDone(event) => Some(event.sequence_number),
            Self::ImageGenerationPartialImage(event) => Some(event.sequence_number),
            Self::McpCallArgumentsDone(event) => Some(event.sequence_number),
            Self::OutputTextAnnotationAdded(event) => Some(event.sequence_number),
            Self::CustomToolCallInputDone(event) => Some(event.sequence_number),
            Self::Unknown(raw) => raw
                .as_object()
                .get("sequence_number")
                .and_then(serde_json::Value::as_u64),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn all_pinned_event_tags_are_recognized() {
        // Derived from specs/openai/openapi.documented.yml (2.3.0).
        let tags = [
            "response.audio.delta",
            "response.audio.done",
            "response.audio.transcript.delta",
            "response.audio.transcript.done",
            "response.code_interpreter_call_code.delta",
            "response.code_interpreter_call_code.done",
            "response.code_interpreter_call.completed",
            "response.code_interpreter_call.in_progress",
            "response.code_interpreter_call.interpreting",
            "response.completed",
            "response.content_part.added",
            "response.content_part.done",
            "response.created",
            "error",
            "response.file_search_call.completed",
            "response.file_search_call.in_progress",
            "response.file_search_call.searching",
            "response.function_call_arguments.delta",
            "response.function_call_arguments.done",
            "response.in_progress",
            "response.failed",
            "response.incomplete",
            "response.output_item.added",
            "response.output_item.done",
            "response.reasoning_summary_part.added",
            "response.reasoning_summary_part.done",
            "response.reasoning_summary_text.delta",
            "response.reasoning_summary_text.done",
            "response.reasoning_text.delta",
            "response.reasoning_text.done",
            "response.refusal.delta",
            "response.refusal.done",
            "response.output_text.delta",
            "response.output_text.done",
            "response.web_search_call.completed",
            "response.web_search_call.in_progress",
            "response.web_search_call.searching",
            "response.image_generation_call.completed",
            "response.image_generation_call.generating",
            "response.image_generation_call.in_progress",
            "response.image_generation_call.partial_image",
            "response.mcp_call_arguments.delta",
            "response.mcp_call_arguments.done",
            "response.mcp_call.completed",
            "response.mcp_call.failed",
            "response.mcp_call.in_progress",
            "response.mcp_list_tools.completed",
            "response.mcp_list_tools.failed",
            "response.mcp_list_tools.in_progress",
            "response.output_text.annotation.added",
            "response.queued",
            "response.custom_tool_call_input.delta",
            "response.custom_tool_call_input.done",
        ];

        let response = serde_json::json!({
            "metadata": {}, "top_logprobs": null, "temperature": 1.0, "top_p": 1.0,
            "model": "gpt-5.4", "tools": [], "tool_choice": "auto", "id": "resp_1",
            "object": "response", "status": "completed", "created_at": 1,
            "completed_at": 2, "error": null, "incomplete_details": null, "output": [],
            "reasoning": {}, "instructions": null, "output_text": "", "usage": null,
            "prompt_cache_options": null, "moderation": null, "parallel_tool_calls": true,
            "conversation": null, "max_output_tokens": null, "truncation": "disabled",
            "previous_response_id": null
        });
        assert_eq!(tags.len(), 53);
        for (sequence_number, tag) in tags.into_iter().enumerate() {
            let mut value = serde_json::json!({
                "type": tag, "sequence_number": sequence_number as u64, "delta": "x",
                "response_id": "resp_1", "output_index": 0, "item_id": "item_1",
                "code": "x", "message": "x", "param": null, "response": response,
                "content_index": 0, "part": {"type":"output_text","text":"x","annotations":[],"logprobs":[]},
                "item": {"type":"message","id":"msg_1","role":"assistant","content":[],"status":"completed"},
                "name": "tool", "arguments": "{}", "summary_index": 0, "text": "x",
                "refusal": "x", "logprobs": [], "partial_image_index": 0,
                "partial_image_b64": "eA==", "annotation_index": 0,
                "annotation": {"type":"file_path","file_id":"file_1","index":0}, "input": "{}"
            });
            if tag.starts_with("response.reasoning_summary_part.") {
                value["part"] = serde_json::json!({"type":"summary_text","text":"x"});
            }
            let event: OpenAIResponsesStreamEvent = serde_json::from_value(value).unwrap();
            assert!(!matches!(event, OpenAIResponsesStreamEvent::Unknown(_)));
        }
    }

    #[test]
    fn unknown_event_round_trips_and_malformed_known_event_errors() {
        let future = serde_json::json!({"type":"response.future.delta","secret":"kept"});
        let event: OpenAIResponsesStreamEvent = serde_json::from_value(future.clone()).unwrap();
        assert_eq!(serde_json::to_value(event).unwrap(), future);
        assert!(serde_json::from_value::<OpenAIResponsesStreamEvent>(
            serde_json::json!({"type":"response.output_text.delta","delta":"missing indexes"})
        )
        .is_err());
    }

    #[test]
    fn queued_example_schema_conflict_is_a_pinned_source_defect() {
        // The pinned x-oaiMeta example omits required Response fields and uses
        // RFC3339 strings where the referenced Response schema requires Unix
        // timestamps. Keep resource/lifecycle decoding strict and make the
        // source contradiction executable instead of silently weakening it.
        let documented_example = serde_json::json!({
            "type": "response.queued",
            "response": {
                "id": "res_123",
                "status": "queued",
                "created_at": "2021-01-01T00:00:00Z",
                "updated_at": "2021-01-01T00:00:00Z"
            },
            "sequence_number": 1
        });
        assert!(serde_json::from_value::<OpenAIResponsesStreamEvent>(documented_example).is_err());
    }
}
