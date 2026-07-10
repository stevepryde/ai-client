use ai_client::openai::responses::{
    ExtendedReasoningEffort, Gpt5_4, ResponseRequest, Temperature,
};

fn main() {
    let _ = ResponseRequest::<Gpt5_4>::builder()
        .temperature(Temperature::new(0.5).unwrap())
        .reasoning(ExtendedReasoningEffort::Low)
        .build();
}
