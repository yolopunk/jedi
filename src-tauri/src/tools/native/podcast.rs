// 播客内置工具

use super::app_handle;
use crate::api::podcast::{
  fetch_episodes, fetch_rss_channel, get_subscriptions, remove_subscription, save_subscription,
};
use crate::tools::{AgentTool, RiskLevel, ToolDeclaration, ToolOutcome, ToolSource};
use async_trait::async_trait;
use serde_json::{json, Value};
use std::sync::Arc;

const GROUP: &str = "podcast";

pub fn tools() -> Vec<Arc<dyn AgentTool>> {
  vec![
    Arc::new(PodcastSubscriptions),
    Arc::new(PodcastEpisodes),
    Arc::new(PodcastSubscribe),
    Arc::new(PodcastUnsubscribe),
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

fn get_str(args: &Value, key: &str) -> Result<String, String> {
  args
    .get(key)
    .and_then(|v| v.as_str())
    .map(|s| s.to_string())
    .ok_or_else(|| format!("缺少参数: {}", key))
}

pub struct PodcastSubscriptions;

#[async_trait]
impl AgentTool for PodcastSubscriptions {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "podcast_subscriptions",
      "列出已订阅的播客",
      json!({ "type": "object", "properties": {} }),
      RiskLevel::Read,
    )
  }

  async fn call(&self, _args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let app = match app_handle() {
      Ok(a) => a,
      Err(e) => return ToolOutcome::error(e),
    };
    match get_subscriptions(app).await {
      Ok(subs) => match serde_json::to_string_pretty(&subs) {
        Ok(s) => ToolOutcome::text(s),
        Err(e) => ToolOutcome::error(format!("序列化失败: {}", e)),
      },
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct PodcastEpisodes;

#[async_trait]
impl AgentTool for PodcastEpisodes {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "podcast_episodes",
      "获取指定播客 RSS 的剧集列表",
      json!({
        "type": "object",
        "properties": {
          "rss_url": { "type": "string", "description": "播客的 RSS 地址" }
        },
        "required": ["rss_url"]
      }),
      RiskLevel::Read,
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let rss_url = match get_str(&args, "rss_url") {
      Ok(u) => u,
      Err(e) => return ToolOutcome::error(e),
    };
    match fetch_episodes(rss_url).await {
      Ok(eps) => match serde_json::to_string_pretty(&eps) {
        Ok(s) => ToolOutcome::text(s),
        Err(e) => ToolOutcome::error(format!("序列化失败: {}", e)),
      },
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct PodcastSubscribe;

#[async_trait]
impl AgentTool for PodcastSubscribe {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "podcast_subscribe",
      "通过 RSS 地址订阅一个播客",
      json!({
        "type": "object",
        "properties": {
          "rss_url": { "type": "string", "description": "播客的 RSS 地址" }
        },
        "required": ["rss_url"]
      }),
      RiskLevel::Write,
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let rss_url = match get_str(&args, "rss_url") {
      Ok(u) => u,
      Err(e) => return ToolOutcome::error(e),
    };
    let app = match app_handle() {
      Ok(a) => a,
      Err(e) => return ToolOutcome::error(e),
    };
    let sub = match fetch_rss_channel(rss_url).await {
      Ok(s) => s,
      Err(e) => return ToolOutcome::error(format!("解析 RSS 失败: {}", e)),
    };
    let title = sub.title.clone();
    match save_subscription(app, sub).await {
      Ok(_) => ToolOutcome::text(format!("已订阅: {}", title)),
      Err(e) => ToolOutcome::error(e),
    }
  }
}

pub struct PodcastUnsubscribe;

#[async_trait]
impl AgentTool for PodcastUnsubscribe {
  fn declaration(&self) -> ToolDeclaration {
    decl(
      "podcast_unsubscribe",
      "取消订阅指定 RSS 地址的播客",
      json!({
        "type": "object",
        "properties": {
          "rss_url": { "type": "string", "description": "要取消订阅的播客 RSS 地址" }
        },
        "required": ["rss_url"]
      }),
      RiskLevel::Write,
    )
  }

  async fn call(&self, args: Value, _snapshot: Option<String>) -> ToolOutcome {
    let rss_url = match get_str(&args, "rss_url") {
      Ok(u) => u,
      Err(e) => return ToolOutcome::error(e),
    };
    let app = match app_handle() {
      Ok(a) => a,
      Err(e) => return ToolOutcome::error(e),
    };
    match remove_subscription(app, rss_url).await {
      Ok(_) => ToolOutcome::text("已取消订阅".to_string()),
      Err(e) => ToolOutcome::error(e),
    }
  }
}
