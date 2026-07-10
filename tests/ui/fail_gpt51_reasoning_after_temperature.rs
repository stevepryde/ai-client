use ai_client::openai::responses::{
    Gpt5_1, Gpt5_1ReasoningEffort, ResponseRequest, Temperature,
};

fn main() {
    let _ = ResponseRequest::<Gpt5_1>::builder()
        .temperature(Temperature::new(0.5).unwrap())
        .reasoning(Gpt5_1ReasoningEffort::Low)
        .build();
}
