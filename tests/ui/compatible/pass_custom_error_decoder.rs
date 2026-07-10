use ai_client::openai_compatible::{
    CompatibleErrorDecoder, CompatibleErrorDetails, CustomDialect, OpenAICompatibleClient,
};

struct Decoder;
impl CompatibleErrorDecoder for Decoder {
    fn decode(&self, _body: &[u8]) -> CompatibleErrorDetails {
        CompatibleErrorDetails::new("decoded")
    }
}

fn main() {
    let _ = OpenAICompatibleClient::<CustomDialect>::builder()
        .error_decoder(Decoder);
}
