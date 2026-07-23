use aiway_plugin::PluginContext;
use aiway_plugin::http::{self, request};
use aiway_plugin::serde_json::{Value, from_value, json};
use aiway_plugin::{Plugin, PluginError, PluginInfo, Version, async_trait, export_wasm};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::sync::RwLock;

/// 路径重写插件
pub struct RewritePathPlugin {
    cached_pattern: RwLock<String>,
    cached_regex: RwLock<Option<Regex>>,
}

impl RewritePathPlugin {
    pub fn new() -> Self {
        Self {
            cached_pattern: RwLock::new(String::new()),
            cached_regex: RwLock::new(None),
        }
    }

    fn get_regex(&self, pattern: &str) -> Result<Regex, PluginError> {
        // 先读锁检查缓存
        {
            let cached_pattern = self.cached_pattern.read().unwrap();
            if *cached_pattern == pattern {
                return Ok(self.cached_regex.read().unwrap().as_ref().unwrap().clone());
            }
        }
        // 缓存未命中，写锁更新
        let new_regex = Regex::new(pattern).map_err(|e| {
            PluginError::ExecuteError(format!("Invalid regex pattern '{}': {}", pattern, e))
        })?;
        *self.cached_pattern.write().unwrap() = pattern.to_string();
        *self.cached_regex.write().unwrap() = Some(new_regex.clone());
        Ok(new_regex)
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRule {
    /// 匹配模式（正则），例如：/api/v1/(.*)
    pub pattern: String,
    /// 替换字符串，如：/api/v2/$1
    pub replacement: String,
}

#[async_trait]
impl Plugin for RewritePathPlugin {
    fn name(&self) -> &str {
        "rewrite-path"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: json!({
                "pattern": "/api/(.*)",
                "replacement": "/$1"
            }),
            description: "路径重写插件".to_string(),
            readme: None,
        }
    }

    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let rule: RewriteRule = from_value(config.clone())
            .map_err(|e| PluginError::ExecuteError(format!("Failed to parse config: {}", e)))?;

        let regex = self.get_regex(&rule.pattern)?;
        let original_path = head.uri.path();

        // 快速路径：不匹配则跳过
        if !regex.is_match(original_path) {
            return Ok(());
        }

        let rewritten = regex.replace(original_path, &rule.replacement);

        // 重建 path_and_query
        let new_pq = match head.uri.query() {
            Some(q) => format!("{}?{}", rewritten, q),
            None => rewritten.into_owned(),
        };

        let mut parts = head.uri.clone().into_parts();
        parts.path_and_query = Some(
            new_pq
                .parse()
                .map_err(|e| PluginError::ExecuteError(format!("Invalid path: {}", e)))?,
        );
        head.uri = http::Uri::from_parts(parts)
            .map_err(|e| PluginError::ExecuteError(format!("URI rebuild failed: {}", e)))?;

        Ok(())
    }
}

// 导出 WASM 插件
export_wasm!(RewritePathPlugin);
