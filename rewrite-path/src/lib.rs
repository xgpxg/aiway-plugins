use aiway_plugin::protocol::context::http::request;
use aiway_plugin::protocol::context::{HttpContext, RequestExt};
use aiway_plugin::serde_json::{Value, json};
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version, serde_json,
};
use regex::Regex;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::sync::{LazyLock, Mutex};

/// 路径重写插件
pub struct RewritePathPlugin;

impl RewritePathPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RewriteRule {
    /// 匹配模式（正则），例如：/api/*
    pub pattern: String,
    /// 替换字符串，如：/$1
    pub replacement: String,
}

static REGEX_CACHE: LazyLock<Mutex<HashMap<String, Regex>>> =
    LazyLock::new(|| Mutex::new(HashMap::new()));

#[async_trait]
impl Plugin for RewritePathPlugin {
    fn name(&self) -> &'static str {
        "rewrite-path"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: json!({
                "pattern": "/api/*",
                "replacement": "/$1"
            }),
            description: "Rewrite path plugin".to_string(),
        }
    }

    // 实现插件逻辑
    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        _: &mut HttpContext,
    ) -> Result<(), PluginError> {
        let rule: RewriteRule = serde_json::from_value(config.clone()).map_err(|e| {
            PluginError::ExecuteError(format!("Failed to parse rewrite rules: {}", e))
        })?;

        let path = head.get_path();

        let regex = {
            let mut cache = REGEX_CACHE.lock().unwrap();
            if let Some(cached_regex) = cache.get(&rule.pattern) {
                cached_regex.clone()
            } else {
                let new_regex = Regex::new(&rule.pattern).map_err(|e| {
                    PluginError::ExecuteError(format!("Failed to parse rewrite rules: {}", e))
                })?;
                cache.insert(rule.pattern.clone(), new_regex.clone());
                new_regex
            }
        };

        let rewritten_path = if regex.is_match(&path) {
            regex.replace_all(&path, &rule.replacement).to_string()
        } else {
            path
        };

        head.set_path(&rewritten_path);
        Ok(Default::default())
    }
}

// 导出插件
export!(RewritePathPlugin);
