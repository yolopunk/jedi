use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TaskSpec {
  pub id: String,
  pub description: String,
  pub prompt: String,
  pub tools: Vec<String>,
  pub model: Option<String>,
  pub abort_on_error: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerStatus {
  pub id: String,
  pub task_id: Option<String>,
  pub description: String,
  pub status: WorkerStatusKind,
  pub progress: Option<String>,
  pub started_at: Option<i64>,
  pub result: Option<String>,
  pub error: Option<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "lowercase")]
pub enum WorkerStatusKind {
  Idle,
  Running,
  Completed,
  Failed,
  Stopped,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct WorkerResult {
  pub worker_id: String,
  pub task_id: String,
  pub status: WorkerStatusKind,
  pub result: Option<String>,
  pub error: Option<String>,
  pub duration_ms: Option<u64>,
}
