//! Optional, observational progress. Completion is proved by the operation result,
//! never by a progress message. Counts are decimal strings across the JS boundary.
use serde::Serialize;

#[derive(Clone, Debug, Serialize)]
pub struct OperationProgress {
    pub stage: String,
    pub completed: Option<String>,
    pub total: Option<String>,
    pub unit: Option<String>,
}

pub type ProgressSink<'a> = &'a dyn Fn(OperationProgress);

pub fn stage(sink: ProgressSink<'_>, name: &str) {
    sink(OperationProgress {
        stage: name.into(),
        completed: None,
        total: None,
        unit: None,
    });
}

pub fn bytes(sink: ProgressSink<'_>, name: &str, completed: u64, total: u64) {
    sink(OperationProgress {
        stage: name.into(),
        completed: Some(completed.to_string()),
        total: Some(total.to_string()),
        unit: Some("bytes".into()),
    });
}
