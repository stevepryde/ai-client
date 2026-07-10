use ai_client::openai::{
    create_response::OpenAIResponsesInput,
    responses::{
        DynamicOpenAIModel, DynamicResponseRequest, ResponseModelCapabilities,
        ResponseModelCapabilitiesCatalog, StaticResponseModelCapabilitiesCatalog,
        ValidationMode,
    },
};

fn assert_catalog<C: ResponseModelCapabilitiesCatalog>(catalog: &C) {
    let _ = catalog.version();
}

fn main() {
    let mut catalog = StaticResponseModelCapabilitiesCatalog::new("local-v1");
    catalog.insert("local-model", ResponseModelCapabilities::new());
    assert_catalog(&catalog);
    let request = DynamicResponseRequest::builder(DynamicOpenAIModel::new("local-model").unwrap())
        .input(OpenAIResponsesInput::Text("hello".into()))
        .validation(ValidationMode::Strict)
        .catalog(catalog)
        .build();
    assert!(request.is_ok());
}
