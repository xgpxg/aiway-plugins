use aiway_plugin::protocol::gateway::HttpContext;
use aiway_plugin::serde_json::{Value, json};
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version, serde_json,
};
use aiway_plugin::protocol::model::Provider;

pub struct BaiLianModelRequestWrapper;

impl BaiLianModelRequestWrapper {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for BaiLianModelRequestWrapper {
    fn name(&self) -> &'static str {
        "BaiLianModelRequestWrapper"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "阿里百炼平台模型接口请求适配".to_string(),
        }
    }

    // 实现插件逻辑
    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        let provider = context.request.get_state::<Provider>("provider").map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        if provider.is_none(){
            return Err(PluginError::ExecuteError("provider is not found".to_string()));
        }
        let provider = provider.unwrap();

        let body = serde_json::from_slice::<Value>(context.request.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let model = body["model"].as_str().ok_or(PluginError::ExecuteError(
            "model field is not found".to_string(),
        ))?;
        let prompt = body["prompt"].as_str().ok_or(PluginError::ExecuteError(
            "prompt field is not found".to_string(),
        ))?;
        match provider.api_url.as_str() {
            // 文生图
            p if p.ends_with("/api/v1/services/aigc/multimodal-generation/generation") => {
                let result = json!({
                    "model": model,
                    "input": {
                        "messages": [
                            {
                                "role": "user",
                                "content": [
                                    {
                                        "text": prompt
                                    }
                                ]
                            }
                        ]
                    },
                });
                Ok(result)
            }
            _ => Ok(body),
        }
    }
}

// 导出插件
export!(BaiLianModelRequestWrapper);
