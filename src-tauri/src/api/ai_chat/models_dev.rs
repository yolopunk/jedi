// Models.dev API 集成模块
// 提供从 https://models.dev/api.json 获取模型信息的功能

use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::Mutex;
use std::time::{Duration, SystemTime};
use tauri::State;

// ========== Models.dev 数据结构 ==========

/// 模型模态能力
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Modalities {
  pub input: Vec<String>,
  pub output: Vec<String>,
}

/// 超过 200k token 的成本信息（嵌套结构）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextOver200kCost {
  pub input: Option<f64>,
  pub output: Option<f64>,
  pub cache_read: Option<f64>,
  pub cache_write: Option<f64>,
  #[serde(flatten)]
  pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 模型成本信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelCost {
  pub input: Option<f64>,
  pub output: Option<f64>,
  pub cache_read: Option<f64>,
  pub cache_write: Option<f64>,
  pub reasoning: Option<f64>,
  pub input_audio: Option<f64>,
  pub output_audio: Option<f64>,
  pub context_over_200k: Option<ContextOver200kCost>,
  #[serde(flatten)]
  pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 模型限制信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelLimits {
  pub context: Option<u32>,
  pub input: Option<u32>,
  pub output: Option<u32>,
}

/// 交错输出配置（可能是布尔值或对象）
/// - `true` 表示启用交错输出
/// - `{"field": "reasoning_content"}` 表示使用指定字段
#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(untagged)]
pub enum InterleavedConfig {
  Enabled(bool),
  WithField { field: String },
}

/// Provider 引用信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderRef {
  pub npm: Option<String>,
  #[serde(flatten)]
  pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 单个模型信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevModel {
  pub id: String,
  pub name: String,
  pub family: Option<String>,
  pub attachment: bool,
  pub reasoning: bool,
  pub tool_call: bool,
  pub structured_output: Option<bool>,
  pub temperature: Option<bool>,
  pub knowledge: Option<String>,
  pub release_date: Option<String>,
  pub last_updated: Option<String>,
  pub modalities: Modalities,
  pub open_weights: bool,
  pub cost: Option<ModelCost>,
  pub limit: Option<ModelLimits>,
  pub interleaved: Option<InterleavedConfig>,
  pub provider: Option<ProviderRef>,
  pub status: Option<String>,
  #[serde(flatten)]
  pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// 提供商信息
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ModelsDevProvider {
  pub id: String,
  pub name: String,
  pub api: Option<String>,
  pub doc: Option<String>,
  pub npm: Option<String>,
  pub env: Option<Vec<String>>,
  pub models: HashMap<String, ModelsDevModel>,
  #[serde(flatten)]
  pub extra: std::collections::HashMap<String, serde_json::Value>,
}

/// Models.dev API 响应（顶级对象）
pub type ModelsDevResponse = HashMap<String, ModelsDevProvider>;

// ========== 缓存结构 ==========

#[derive(Debug, Clone)]
struct CacheEntry {
  data: ModelsDevResponse,
  timestamp: SystemTime,
}

/// Models.dev 管理器状态
pub struct ModelsDevManagerState {
  cache: Mutex<Option<CacheEntry>>,
  client: Client,
}

impl ModelsDevManagerState {
  pub fn new() -> Self {
    Self {
      cache: Mutex::new(None),
      client: Client::builder()
        .timeout(Duration::from_secs(30))
        .build()
        .unwrap_or_else(|_| Client::new()),
    }
  }
}

// 默认缓存有效期：1 小时
const CACHE_TTL: Duration = Duration::from_secs(3600);
const MODELS_DEV_API_URL: &str = "https://models.dev/api.json";

// ========== Tauri Commands ==========

/// 从 models.dev 获取所有提供商和模型信息
#[tauri::command]
pub async fn fetch_models_dev(
  state: State<'_, ModelsDevManagerState>,
  force_refresh: Option<bool>,
) -> Result<ModelsDevResponse, String> {
  let force = force_refresh.unwrap_or(false);

  // 检查缓存
  {
    let cache = state.cache.lock().map_err(|e| e.to_string())?;
    if !force {
      if let Some(entry) = cache.as_ref() {
        if let Ok(elapsed) = entry.timestamp.elapsed() {
          if elapsed < CACHE_TTL {
            return Ok(entry.data.clone());
          }
        }
      }
    }
  }

  // 从 API 获取新数据
  let response = state
    .client
    .get(MODELS_DEV_API_URL)
    .send()
    .await
    .map_err(|e| format!("Failed to fetch models.dev API: {}", e))?;

  if !response.status().is_success() {
    return Err(format!(
      "Models.dev API returned error: {}",
      response.status()
    ));
  }

  let data: ModelsDevResponse = response
    .json()
    .await
    .map_err(|e| format!("Failed to parse models.dev response: {}", e))?;

  // 更新缓存
  {
    let mut cache = state.cache.lock().map_err(|e| e.to_string())?;
    *cache = Some(CacheEntry {
      data: data.clone(),
      timestamp: SystemTime::now(),
    });
  }

  Ok(data)
}

/// 获取指定提供商的信息
#[tauri::command]
pub async fn get_models_dev_provider(
  state: State<'_, ModelsDevManagerState>,
  provider_id: String,
) -> Result<Option<ModelsDevProvider>, String> {
  let data = fetch_models_dev(state, Some(false)).await?;
  Ok(data.get(&provider_id).cloned())
}

/// 搜索模型（按名称或 ID）
#[tauri::command]
pub async fn search_models_dev(
  state: State<'_, ModelsDevManagerState>,
  query: String,
  provider_filter: Option<String>,
) -> Result<Vec<(String, ModelsDevModel)>, String> {
  let data = fetch_models_dev(state, Some(false)).await?;
  let query_lower = query.to_lowercase();
  let mut results = Vec::new();

  for (provider_id, provider) in data {
    if let Some(filter) = &provider_filter {
      if provider_id.to_lowercase() != filter.to_lowercase() {
        continue;
      }
    }

    for (model_id, model) in provider.models {
      if model_id.to_lowercase().contains(&query_lower)
        || model.name.to_lowercase().contains(&query_lower)
      {
        results.push((provider_id.clone(), model));
      }
    }
  }

  Ok(results)
}

/// 应用提供商类型到 models.dev ID 的映射
fn map_app_provider_to_models_dev(app_provider: &str) -> Vec<String> {
  match app_provider.to_lowercase().as_str() {
    "openai" => vec!["openai".to_string()],
    "anthropic" => vec!["cloudflare-ai-gateway".to_string()],
    "google" => vec!["inference".to_string()],
    "ollama" => vec!["ollama-cloud".to_string()],
    "cohere" => vec!["cohere".to_string()],
    _ => vec![app_provider.to_string()],
  }
}

/// 检查模型 ID 是否匹配目标提供商
fn model_matches_provider(model_id: &str, target_provider: &str) -> bool {
  let target_lower = target_provider.to_lowercase();
  let model_id_lower = model_id.to_lowercase();

  match target_lower.as_str() {
    "anthropic" => model_id_lower.starts_with("anthropic/"),
    "google" => model_id_lower.starts_with("google/"),
    "openai" => !model_id.contains('/'),
    _ => true,
  }
}

/// 清理模型 ID（移除提供商前缀）
fn clean_model_id(model_id: &str, target_provider: &str) -> String {
  let target_lower = target_provider.to_lowercase();
  match target_lower.as_str() {
    "anthropic" => model_id
      .strip_prefix("anthropic/")
      .unwrap_or(model_id)
      .to_string(),
    "google" => model_id
      .strip_prefix("google/")
      .unwrap_or(model_id)
      .to_string(),
    _ => model_id.to_string(),
  }
}

/// 获取某个提供商的模型列表（智能映射）
#[tauri::command]
pub async fn get_models_for_provider(
  state: State<'_, ModelsDevManagerState>,
  provider_id: String,
) -> Result<Vec<ModelsDevModel>, String> {
  let data = fetch_models_dev(state, Some(false)).await?;
  let mut models = Vec::new();

  let mapped_provider_ids = map_app_provider_to_models_dev(&provider_id);

  for source_provider_id in mapped_provider_ids {
    if let Some(provider) = data.get(&source_provider_id) {
      for (model_id, model) in &provider.models {
        if model_matches_provider(model_id, &provider_id) {
          let mut cloned_model = model.clone();
          cloned_model.id = clean_model_id(model_id, &provider_id);
          models.push(cloned_model);
        }
      }
    }
  }

  Ok(models)
}

/// 获取所有支持的提供商列表（精简信息）
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ProviderSummary {
  pub id: String,
  pub name: String,
  pub api: Option<String>,
  pub doc: Option<String>,
  pub model_count: usize,
}

#[tauri::command]
pub async fn get_models_providers(
  state: State<'_, ModelsDevManagerState>,
) -> Result<Vec<ProviderSummary>, String> {
  let data = fetch_models_dev(state, Some(false)).await?;

  let providers: Vec<ProviderSummary> = data
    .into_iter()
    .map(|(id, provider)| ProviderSummary {
      id,
      name: provider.name,
      api: provider.api,
      doc: provider.doc,
      model_count: provider.models.len(),
    })
    .collect();

  Ok(providers)
}

#[cfg(test)]
mod tests {
  use super::*;

  #[test]
  fn test_modalities_deserialize() {
    let json = r#"{"input": ["text", "image"], "output": ["text"]}"#;
    let modalities: Modalities = serde_json::from_str(json).unwrap();
    assert_eq!(modalities.input.len(), 2);
    assert_eq!(modalities.output.len(), 1);
  }

  #[test]
  fn test_model_limits_deserialize() {
    let json = r#"{"context": 128000, "output": 4096}"#;
    let limits: ModelLimits = serde_json::from_str(json).unwrap();
    assert_eq!(limits.context, Some(128000));
    assert_eq!(limits.output, Some(4096));
  }

  #[test]
  fn test_model_cost_deserialize() {
    let json = r#"{"input": 0.0015, "output": 0.002, "cache_read": 0.00015}"#;
    let cost: ModelCost = serde_json::from_str(json).unwrap();
    assert_eq!(cost.input, Some(0.0015));
    assert_eq!(cost.cache_write, None);
  }
}
