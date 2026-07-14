use ai_client::openai::responses::{
    ExtendedReasoningEffort, Gpt5_2, Temperature,
};

fn main() {
    let _ = Gpt5_2::config()
        .reasoning(ExtendedReasoningEffort::High)
        .temperature(Temperature::new(0.5).unwrap());
}
