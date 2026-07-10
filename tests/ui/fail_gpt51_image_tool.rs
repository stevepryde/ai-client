use ai_client::openai::responses::{
    Gpt5_1, OpenAIImageGenerationTool, ResponseRequest,
};

fn main() {
    let _ = ResponseRequest::<Gpt5_1>::builder()
        .tool(OpenAIImageGenerationTool::default())
        .build();
}
