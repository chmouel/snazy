use serde_json::Value;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct StructuredLog {
    pub level: String,
    pub message: String,
    pub timestamp: Option<String>,
    pub others: Option<String>,
    pub extra_fields: Vec<(String, String)>,
    pub stacktrace: Option<String>,
    pub raw_json: Option<Value>,
    pub kail_prefix: Option<String>,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct KubectlEvent {
    pub last_seen: String,
    pub type_: String,
    pub reason: String,
    pub object: String,
    pub message: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParsedLine {
    Structured(StructuredLog),
    Raw(String),
    KubectlHeader,
    KubectlEvent(KubectlEvent),
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct RenderedLog {
    pub level: String,
    pub timestamp: String,
    pub others: String,
    pub message: String,
    pub stacktrace: Option<String>,
}
