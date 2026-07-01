// 壁纸内置工具

use super::app_handle;
use crate::api::wallpapers::{get_current_wallpaper, get_wallpapers, set_desktop_wallpaper};
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolSource};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

const GROUP: &str = "wallpaper";

pub fn tools() -> Vec<Arc<dyn AgentTool>> {
  vec![
    Arc::new(WallpaperList),
    Arc::new(WallpaperCurrent),
    Arc::new(WallpaperSet),
  ]
}

fn decl(name: &str, description: &str, schema: Value, risk: RiskLevel) -> ToolDeclaration {
  ToolDeclaration {
    name: name.into(),
    description: description.into(),
    input_schema: schema,
    risk,
    source: ToolSource::Native,
    group: GROUP.into(),
  }
}

pub struct WallpaperList;

#[async_trait]
impl AgentTool for WallpaperList {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "wallpaper_list",
      "列出可用的知识壁纸",
      json!({ "type": "object", "properties": {} }),
      RiskLevel::Read,
    )
  }

  async fn call(&self, _args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let app = match app_handle() {
      Ok(a) => a,
      Err(e) => return ToolOutcome::error(e),
    };
    match get_wallpapers(app).await {
      Ok(items) => match serde_json::to_string_pretty(&items) {
        Ok(s) => ToolOutcome::text(s),
        Err(e) => ToolOutcome::error(format!("序列化失败: {}", e)),
      },
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct WallpaperCurrent;

#[async_trait]
impl AgentTool for WallpaperCurrent {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "wallpaper_current",
      "获取当前桌面壁纸路径",
      json!({ "type": "object", "properties": {} }),
      RiskLevel::Read,
    )
  }

  async fn call(&self, _args: Value, _snapshot: Option<String>) -> ToolOutcome {
    match get_current_wallpaper().await {
      Ok(path) => ToolOutcome::text(path),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct WallpaperSet;

#[async_trait]
impl AgentTool for WallpaperSet {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "wallpaper_set",
      "把指定 URL/路径的图片设为桌面壁纸",
      json!({
        "type": "object",
        "properties": {
          "url": { "type": "string", "description": "壁纸图片的 URL 或本地路径" }
        },
        "required": ["url"]
      }),
      RiskLevel::Write,
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let url = match args.get("url").and_then(|v| v.as_str()) {
      Some(u) => u.to_string(),
      None => return ToolOutcome::error("缺少参数: url".to_string()),
    };
    let app = match app_handle() {
      Ok(a) => a,
      Err(e) => return ToolOutcome::error(e),
    };
    match set_desktop_wallpaper(app, url.clone(), None).await {
      Ok(()) => ToolOutcome::text(format!("已设置壁纸: {}", url)),
      Err(e) => ToolOutcome::error(e),
    }
  }
}
