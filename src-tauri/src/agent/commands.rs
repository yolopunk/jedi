use crate::agent::pool::AgentPool;
use crate::agent::types::{TaskSpec, WorkerStatus};
use tauri::State;
use tokio::sync::Mutex;

pub struct PoolState(pub Mutex<AgentPool>);

#[tauri::command]
pub fn get_pool_status(state: State<PoolState>) -> Vec<WorkerStatus> {
  // Blocking lock for sync access
  state.0.blocking_lock().status()
}

#[tauri::command]
pub fn schedule_task(state: State<PoolState>, task: TaskSpec) -> Result<String, String> {
  state.0.blocking_lock().schedule(task)
}

#[tauri::command]
pub async fn stop_worker(state: State<'_, PoolState>, worker_id: String) -> Result<(), String> {
  state.0.lock().await.stop(&worker_id).await
}

#[tauri::command]
pub async fn remove_worker(state: State<'_, PoolState>, worker_id: String) -> Result<(), String> {
  state.0.lock().await.remove(&worker_id).await?;
  Ok(())
}

#[tauri::command]
pub fn init_pool(state: State<PoolState>, max_workers: usize) -> Result<(), String> {
  let mut pool = state.0.blocking_lock();
  *pool = AgentPool::new(max_workers);
  Ok(())
}
