// 系统信息内置工具

use super::app_handle;
use crate::api::os::{get_os_info, SystemState};
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolSource};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;
use tauri::Manager;

pub fn tools() -> Vec<Arc<dyn AgentTool>> {
  vec![Arc::new(SystemInfo)]
}

pub struct SystemInfo;

#[async_trait]
impl AgentTool for SystemInfo {
  fn declaration(&self) -> ToolDeclaration {
    ToolDeclaration {
      name: "system_info".into(),
      description: "获取系统信息（操作系统、版本、主机名等）".into(),
      input_schema: json!({ "type": "object", "properties": {} }),
      risk: RiskLevel::Read,
      source: ToolSource::Native,
      group: "system".into(),
    }
  }

  async fn call(&self, _args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let app = match app_handle() {
      Ok(a) => a,
      Err(e) => return ToolOutcome::error(e),
    };
    let state = app.state::<SystemState>();
    match get_os_info(state).await {
      Ok(info) => match serde_json::to_string_pretty(&info) {
        Ok(s) => ToolOutcome::text(s),
        Err(e) => ToolOutcome::error(format!("序列化失败: {}", e)),
      },
      Err(e) => ToolOutcome::error(e),
    }
  }
}
