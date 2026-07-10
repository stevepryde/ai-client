use base64::prelude::*;

pub trait IntoQuery {
    fn into_query(self) -> Vec<(String, String)>;
}

pub fn base64_encode(data: &[u8]) -> String {
    BASE64_STANDARD.encode(data)
}

#[allow(dead_code)]
pub fn base64_decode(data: &str) -> Result<Vec<u8>, base64::DecodeError> {
    BASE64_STANDARD.decode(data.as_bytes())
}
