use thiserror::Error;

#[derive(Error, Debug)]
pub enum ErrorKind {
    #[error("Tracing subscriber error: {0}")]
    TracingSubscriberError(String),
    #[error("OpenTelemetry error: {0}")]
    OpenTelemetryError(String),
}

#[derive(Error, Debug)]
#[error("{kind}")]
pub struct WafError {
    pub kind: ErrorKind,
}

impl WafError {
    pub fn new(kind: ErrorKind) -> Self {
        Self { kind }
    }
}
