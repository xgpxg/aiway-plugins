use aiway_plugin::protocol::gateway::HttpContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version};
use serde::{Deserialize, Serialize};

/// Header操作插件
///
/// 该插件支持添加、删除和修改HTTP请求头
pub struct HeaderOperatorPlugin;

impl HeaderOperatorPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HeaderConfig {
    /// 请求头操作配置
    #[serde(default)]
    pub request_headers: RequestHeaderConfig,

    /// 响应头操作配置
    #[serde(default)]
    pub response_headers: ResponseHeaderConfig,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct RequestHeaderConfig {
    /// 要添加的请求头部信息
    #[serde(default)]
    pub add_headers: std::collections::HashMap<String, String>,

    /// 要移除的请求头部名称列表
    #[serde(default)]
    pub remove_headers: Vec<String>,
}

#[derive(Debug, Clone, Serialize, Deserialize, Default)]
pub struct ResponseHeaderConfig {
    /// 要添加的响应头部信息
    #[serde(default)]
    pub add_headers: std::collections::HashMap<String, String>,

    /// 要移除的响应头部名称列表
    #[serde(default)]
    pub remove_headers: Vec<String>,
}

impl Default for HeaderConfig {
    fn default() -> Self {
        Self {
            request_headers: RequestHeaderConfig::default(),
            response_headers: ResponseHeaderConfig::default(),
        }
    }
}

#[async_trait]
impl Plugin for HeaderOperatorPlugin {
    fn name(&self) -> &'static str {
        "HeaderOperator"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: serde_json::to_value(HeaderConfig::default()).unwrap_or_default(),
            description: "新增或移除HTTP头".to_string(),
        }
    }

    async fn execute(&self, context: &HttpContext, config: &Value) -> Result<Value, PluginError> {
        // 解析配置
        let header_config: HeaderConfig = serde_json::from_value(config.clone()).map_err(|e| {
            PluginError::ExecuteError(format!("Failed to parse header config: {}", e))
        })?;

        // 处理请求头
        for header_name in &header_config.request_headers.remove_headers {
            context.request.remove_header(header_name);
        }

        for (key, value) in &header_config.request_headers.add_headers {
            context.request.insert_header(key, value);
        }

        // 处理响应头
        for header_name in &header_config.response_headers.remove_headers {
            context.response.remove_header(header_name);
        }

        for (key, value) in &header_config.response_headers.add_headers {
            context.response.insert_header(key, value);
        }

        Ok(Default::default())
    }
}

// 导出插件
export!(HeaderOperatorPlugin);
