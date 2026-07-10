use std::fmt;

use serde::{Deserialize, Serialize, Serializer};
use serde_json::{Map, Value};

/// A forward-compatible tagged object whose complete semantic JSON is retained.
///
/// The raw body is intentionally omitted from `Debug` because future variants
/// can contain prompts, tool output, credentials, or other sensitive data.
#[derive(Clone, PartialEq)]
pub struct RawTaggedValue {
    tag: String,
    raw: Map<String, Value>,
}

impl RawTaggedValue {
    pub(crate) fn from_map(tag: String, raw: Map<String, Value>) -> Self {
        Self { tag, raw }
    }

    pub fn tag(&self) -> &str {
        &self.tag
    }

    pub fn as_object(&self) -> &Map<String, Value> {
        &self.raw
    }

    pub fn into_value(self) -> Value {
        Value::Object(self.raw)
    }
}

impl fmt::Debug for RawTaggedValue {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_struct("RawTaggedValue")
            .field("tag", &self.tag)
            .field("raw", &"[redacted]")
            .finish()
    }
}

impl Serialize for RawTaggedValue {
    fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
    where
        S: Serializer,
    {
        self.raw.serialize(serializer)
    }
}

pub(crate) fn deserialize_payload<T, E>(mut raw: Map<String, Value>) -> Result<T, E>
where
    T: for<'de> Deserialize<'de>,
    E: serde::de::Error,
{
    raw.remove("type");
    serde_json::from_value(Value::Object(raw)).map_err(E::custom)
}

pub(crate) fn serialize_payload<S, T>(
    tag: &'static str,
    payload: &T,
    serializer: S,
) -> Result<S::Ok, S::Error>
where
    S: Serializer,
    T: Serialize,
{
    let value = serde_json::to_value(payload).map_err(serde::ser::Error::custom)?;
    let Value::Object(mut object) = value else {
        return Err(serde::ser::Error::custom(
            "tagged payload must serialize as an object",
        ));
    };
    object.insert("type".into(), Value::String(tag.into()));
    object.serialize(serializer)
}

macro_rules! lossless_tagged_enum {
    (
        $(#[$meta:meta])*
        pub enum $name:ident {
            $( $variant:ident($payload:ty) => $tag:literal, )+
            @unknown
        }
    ) => {
        $(#[$meta])*
        pub enum $name {
            $( $variant($payload), )+
            Unknown($crate::openai::responses::tagged::RawTaggedValue),
        }

        impl serde::Serialize for $name {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
            where
                S: serde::Serializer,
            {
                match self {
                    $(
                        Self::$variant(payload) =>
                            $crate::openai::responses::tagged::serialize_payload(
                                $tag, payload, serializer,
                            ),
                    )+
                    Self::Unknown(raw) => raw.serialize(serializer),
                }
            }
        }

        impl<'de> serde::Deserialize<'de> for $name {
            fn deserialize<D>(deserializer: D) -> Result<Self, D::Error>
            where
                D: serde::Deserializer<'de>,
            {
                let value = serde_json::Value::deserialize(deserializer)?;
                let serde_json::Value::Object(raw) = value else {
                    return Err(serde::de::Error::custom("tagged value must be an object"));
                };
                let tag = raw
                    .get("type")
                    .and_then(serde_json::Value::as_str)
                    .ok_or_else(|| serde::de::Error::missing_field("type"))?
                    .to_owned();
                match tag.as_str() {
                    $(
                        $tag => $crate::openai::responses::tagged::deserialize_payload(raw)
                            .map(Self::$variant),
                    )+
                    _ => Ok(Self::Unknown(
                        $crate::openai::responses::tagged::RawTaggedValue::from_map(tag, raw),
                    )),
                }
            }
        }
    };
}

pub(crate) use lossless_tagged_enum;

pub type ExtraFields = Map<String, Value>;
