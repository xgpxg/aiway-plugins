use aiway_plugin::protocol::gateway::HttpContext;
use aiway_plugin::protocol::model::Provider;
use aiway_plugin::serde_json::{Value, json};
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version, serde_json,
};

pub struct BaiLianModelResponseWrapper;

impl BaiLianModelResponseWrapper {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for BaiLianModelResponseWrapper {
    fn name(&self) -> &'static str {
        "BaiLianModelResponseWrapper"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "阿里百炼平台模型接口响应适配".to_string(),
        }
    }

    // 实现插件逻辑
    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        let provider = context
            .request
            .get_state::<Provider>("provider")
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        if provider.is_none() {
            return Err(PluginError::ExecuteError(
                "provider is not found".to_string(),
            ));
        }
        let provider = provider.unwrap();

        let body = serde_json::from_slice::<Value>(context.response.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let image_url = body["output"]["choices"][0]["message"]["content"][0]["image"]
            .as_str()
            .ok_or(PluginError::ExecuteError(
                "image field is not found".to_string(),
            ))?;

        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?
            .as_secs() as u32;
        match provider.api_url.as_str() {
            // 文生图
            p if p.ends_with("/api/v1/services/aigc/multimodal-generation/generation") => {
                let result = json!({
                    "created": ts,
                    "data": [
                        {
                            "url": image_url
                        }
                    ],
                });
                Ok(result)
            }
            _ => Ok(body),
        }
    }
}

// 导出插件
export!(BaiLianModelResponseWrapper);
