use base64::Engine;
use base64::engine::general_purpose::STANDARD;
use aiway_plugin::protocol::gateway::HttpContext;
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version, serde_json,
};
use serde_json::Value;

/// # Aha模型请求参数转换
///
/// # 默认配置
/// 无
///
pub struct AhaModelResponseWrapperPlugin;

impl AhaModelResponseWrapperPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for AhaModelResponseWrapperPlugin {
    fn name(&self) -> &'static str {
        "AhaModelResponseWrapperPlugin"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "Aha模型响应参数转换".to_string(),
        }
    }

    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        if !context.response.is_success() {
            return Ok(Default::default());
        }
        let body = &serde_json::from_slice::<Value>(context.response.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let base64 = body["choices"][0]["message"]["content"][0]["audio_url"]["url"]
            .as_str()
            .ok_or(PluginError::ExecuteError(
                "audio_url field is not found".to_string(),
            ))?;

        let base64 = base64.strip_prefix("data:audio/wav;base64,").unwrap();

        let bytes = STANDARD
            .decode(base64)
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        context.response.set_body(bytes.into());
        context.response.insert_header("Content-Type", "audio/wav");

        Ok(Default::default())
    }
}

// 导出插件
export!(AhaModelResponseWrapperPlugin);
