use ai_client::openai::{
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
        .input_text("hello")
        .validation(ValidationMode::Strict)
        .catalog(catalog)
        .build();
    assert!(request.is_ok());
}
