use std::collections::HashMap;
use std::sync::Arc;

use tokio::sync::mpsc;
use tokio::sync::Mutex;

use super::types::{TaskSpec, WorkerResult, WorkerStatus, WorkerStatusKind};

struct WorkerHandle {
    id: String,
    task_id: Option<String>,
    description: String,
    status: WorkerStatusKind,
    progress: Option<String>,
    started_at: Option<i64>,
    abort_tx: mpsc::Sender<()>,
}

pub struct AgentPool {
    max_workers: usize,
    workers: Arc<Mutex<HashMap<String, WorkerHandle>>>,
}

impl AgentPool {
    pub fn new(max_workers: usize) -> Self {
        Self {
            max_workers,
            workers: Arc::new(Mutex::new(HashMap::new())),
        }
    }

    pub fn status(&self) -> Vec<WorkerStatus> {
        let workers = self.workers.blocking_lock();
        workers
            .values()
            .map(|w| WorkerStatus {
                id: w.id.clone(),
                task_id: w.task_id.clone(),
                description: w.description.clone(),
                status: w.status.clone(),
                progress: w.progress.clone(),
                started_at: w.started_at,
                result: None,
                error: None,
            })
            .collect()
    }

    pub fn schedule(&self, _spec: TaskSpec) -> Result<String, String> {
        let workers = self.workers.blocking_lock();

        // Check if we have room for more workers
        if workers.len() >= self.max_workers {
            return Err("Pool at maximum capacity".to_string());
        }

        // Find an idle worker
        let idle_worker = workers.values().find(|w| matches!(w.status, WorkerStatusKind::Idle));

        match idle_worker {
            Some(_worker) => {
                let worker_id = _worker.id.clone();
                // Note: In a real implementation, we would spawn a task here
                // using spec.id and spec.description from the TaskSpec.
                // For now, we just return the worker_id
                Ok(worker_id)
            }
            None => Err("No available workers".to_string()),
        }
    }

    pub async fn stop(&self, worker_id: &str) -> Result<(), String> {
        let mut workers = self.workers.lock().await;

        let worker = workers
            .get_mut(worker_id)
            .ok_or_else(|| format!("Worker {} not found", worker_id))?;

        if !matches!(worker.status, WorkerStatusKind::Running) {
            return Err(format!("Worker {} is not running", worker_id));
        }

        // Fire-and-forget: send abort signal without waiting for confirmation
        let _ = worker.abort_tx.send(()).await;

        worker.status = WorkerStatusKind::Stopped;

        Ok(())
    }

    pub async fn remove(&self, worker_id: &str) -> Result<WorkerResult, String> {
        let mut workers = self.workers.lock().await;

        let worker = workers
            .remove(worker_id)
            .ok_or_else(|| format!("Worker {} not found", worker_id))?;

        let duration_ms = worker
            .started_at
            .map(|started| (std::time::SystemTime::now()
                .duration_since(std::time::UNIX_EPOCH)
                .unwrap()
                .as_millis() as i64 - started) as u64);

        Ok(WorkerResult {
            worker_id: worker.id.clone(),
            task_id: worker.task_id.clone().unwrap_or_default(),
            status: worker.status.clone(),
            result: None,
            error: None,
            duration_ms,
        })
    }
}
