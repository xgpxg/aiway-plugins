use aiway_plugin::http::request;
use aiway_plugin::PluginContext;
use aiway_plugin::serde_json::{self, json, Value};
use aiway_plugin::{
    async_trait, export_wasm, Plugin, PluginError, PluginInfo, Version,
};
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::RwLock;

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum Strategy {
    /// 按请求路径限流
    Path,
    /// 按客户端 IP 限流
    Ip,
    /// 按指定 Header 值限流
    Header,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RateLimitConfig {
    /// 限流策略
    pub strategy: Strategy,

    /// 时间窗口内最大请求数
    pub max_requests: u64,

    /// 时间窗口（秒），默认 60
    #[serde(default = "default_window")]
    pub window: u64,

    /// Header 策略时指定的 header 名称
    pub header_name: Option<String>,

    /// 客户端 IP 来源 header，默认 X-Forwarded-For
    #[serde(default = "default_ip_header")]
    pub ip_header: String,

    /// 限流响应状态码，默认 429
    #[serde(default = "default_status_code")]
    pub status_code: u16,

    /// 限流响应体，默认 "Too Many Requests"
    #[serde(default = "default_response_body")]
    pub response_body: String,
}

fn default_window() -> u64 {
    60
}
fn default_ip_header() -> String {
    "X-Forwarded-For".to_string()
}
fn default_status_code() -> u16 {
    429
}
fn default_response_body() -> String {
    "Too Many Requests".to_string()
}

impl Default for RateLimitConfig {
    fn default() -> Self {
        Self {
            strategy: Strategy::Ip,
            max_requests: 100,
            window: 60,
            header_name: None,
            ip_header: "X-Forwarded-For".to_string(),
            status_code: 429,
            response_body: "Too Many Requests".to_string(),
        }
    }
}

struct SlidingWindowState {
    records: HashMap<String, Vec<u64>>,
}

pub struct RateLimiterPlugin {
    state: RwLock<SlidingWindowState>,
}

impl RateLimiterPlugin {
    pub fn new() -> Self {
        Self {
            state: RwLock::new(SlidingWindowState {
                records: HashMap::new(),
            }),
        }
    }

    /// 检查是否允许请求通过（滑动窗口算法）
    /// now_ms: 请求时间戳（毫秒），由宿主侧提供
    fn check_rate_limit(
        &self,
        key: &str,
        max_requests: u64,
        window_secs: u64,
        now_ms: u64,
    ) -> bool {
        let window_ms = window_secs * 1000;
        let cutoff = now_ms.saturating_sub(window_ms);

        let mut state = self.state.write().unwrap();
        let timestamps = state
            .records
            .entry(key.to_string())
            .or_insert_with(Vec::new);

        // 移除窗口外的过期记录
        timestamps.retain(|&ts| ts > cutoff);

        // 检查是否超限
        if (timestamps.len() as u64) >= max_requests {
            return false;
        }

        // 记录当前请求
        timestamps.push(now_ms);
        true
    }

    /// 从请求中提取限流 key
    fn extract_key(
        &self,
        config: &RateLimitConfig,
        head: &request::Parts,
    ) -> Result<String, PluginError> {
        match config.strategy {
            Strategy::Path => Ok(head.uri.path().to_string()),
            Strategy::Ip => {
                let ip = head
                    .headers
                    .get(&config.ip_header)
                    .and_then(|v| v.to_str().ok())
                    // X-Forwarded-For: client, proxy1, proxy2
                    .map(|v| v.split(',').next().unwrap_or("").trim().to_string())
                    .or_else(|| {
                        // 回退到 X-Real-IP
                        head.headers
                            .get("X-Real-IP")
                            .and_then(|v| v.to_str().ok())
                            .map(|v| v.to_string())
                    })
                    .filter(|v| !v.is_empty())
                    .ok_or_else(|| {
                        PluginError::ExecuteError(format!(
                            "Cannot determine client IP: '{}' header not found",
                            config.ip_header
                        ))
                    })?;
                Ok(ip)
            }
            Strategy::Header => {
                let header_name = config.header_name.as_deref().ok_or_else(|| {
                    PluginError::ExecuteError(
                        "header_name is required for Header strategy".to_string(),
                    )
                })?;
                let value = head
                    .headers
                    .get(header_name)
                    .and_then(|v| v.to_str().ok())
                    .ok_or_else(|| {
                        PluginError::ExecuteError(format!(
                            "Header '{}' not found for rate limiting",
                            header_name
                        ))
                    })?;
                Ok(format!("{}:{}", header_name, value))
            }
        }
    }
}

#[async_trait]
impl Plugin for RateLimiterPlugin {
    fn name(&self) -> &str {
        "rate-limiter"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: json!({
                "strategy": "path",
                "max_requests": 100,
                "window": 60,
                "ip_header": "X-Forwarded-For",
                "status_code": 429,
                "response_body": "Too Many Requests"
            }),
            description: "请求限流插件，支持按路径/IP/Header 进行滑动窗口限流".to_string(),
            readme: Some(include_str!("../README.md").to_string()),
        }
    }

    /// 在 on_request 阶段进行限流检查
    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let cfg: RateLimitConfig = serde_json::from_value(config.clone()).map_err(|e| {
            PluginError::ExecuteError(format!("Failed to parse rate limit config: {}", e))
        })?;

        let key = self.extract_key(&cfg, head)?;
        // 使用宿主侧提供的请求时间戳（毫秒），避免 WASM 中 SystemTime 不可用的问题
        let now_ms = ctx.request_ts() as u64;

        if !self.check_rate_limit(&key, cfg.max_requests, cfg.window, now_ms) {
            return Err(PluginError::Reject(
                cfg.status_code, cfg.response_body
            ));
        }

        Ok(())
    }
}

export_wasm!(RateLimiterPlugin);
