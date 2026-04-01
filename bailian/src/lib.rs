use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::{json, Value};
use aiway_plugin::Version;
use aiway_plugin::{
    async_trait, export, plugin_version, serde_json, Bytes, Plugin, PluginError, PluginInfo,
};

pub struct BaiLianModelWrapper;

impl BaiLianModelWrapper {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for BaiLianModelWrapper {
    fn name(&self) -> &'static str {
        "BaiLian"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "阿里百炼平台模型接口请求适配".to_string(),
        }
    }

    async fn on_request_body(
        &self,
        _config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        let provider = ctx.get_proxy_model_provider();

        if provider.is_none() {
            return Err(PluginError::ExecuteError(
                "provider is not found".to_string(),
            ));
        }

        let provider = provider.unwrap();

        let old_body = serde_json::from_slice::<Value>(body.as_ref().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let model = old_body["model"].as_str().ok_or(PluginError::ExecuteError(
            "model field is not found".to_string(),
        ))?;
        let prompt = old_body["prompt"]
            .as_str()
            .ok_or(PluginError::ExecuteError(
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

                *body = Some(
                    serde_json::to_vec(&result)
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                        .into(),
                );
            }
            _ => {}
        }
        Ok(())
    }
}

// 导出插件
export!(BaiLianModelWrapper);
