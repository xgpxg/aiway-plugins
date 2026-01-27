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
pub struct AhaModelRequestWrapperPlugin;

impl AhaModelRequestWrapperPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for AhaModelRequestWrapperPlugin {
    fn name(&self) -> &'static str {
        "aha-mi"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "Aha模型请求参数转换".to_string(),
        }
    }

    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        let body = &serde_json::from_slice::<Value>(context.request.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let model = body["model"].as_str().ok_or(PluginError::ExecuteError(
            "model field is not found".to_string(),
        ))?;

        let result = match model {
            m if m.starts_with("minicpm4")
                || m.starts_with("qwen2.5vl")
                || m.starts_with("qwen3")
                || m.eq("deepseek-ocr")
                || m.eq("hunyuan-ocr")
                || m.eq("paddleocr-vl") =>
            {
                body.clone()
            }
            m if m.eq("rmbg2.0") => {
                let image = body["image"].as_str().ok_or(PluginError::ExecuteError(
                    "input field is not found".to_string(),
                ))?;
                serde_json::json!({
                    "model": "rmbg2.0",
                    "messages": [{
                        "role": "user",
                        "content": [
                            {
                                "type": "image",
                                "image_url": {
                                    "url": image
                                }
                            }
                        ]}
                    ]
                })
            }
            m if m.starts_with("voxcpm") => {
                let input = body["input"].as_str().ok_or(PluginError::ExecuteError(
                    "input field is not found".to_string(),
                ))?;
                let voice = body["voice"].as_str().ok_or(PluginError::ExecuteError(
                    "voice field is not found".to_string(),
                ))?;
                let voice_text = body["voice_text"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "voice_text field is not found".to_string(),
                    ))?;
                serde_json::json!({
                        "model": model,
                        "messages": [
                            {
                                "role": "user",
                                "content": [
                                    {
                                        "type": "audio",
                                        "audio_url": {
                                            "url": voice
                                        }
                                    },
                                    {
                                        "type": "text",
                                        "text": input
                                    }
                                ]
                            }
                        ],
                        "metadata": {
                            "prompt_text": voice_text
                        }
                    }
                )
            }
            m if m.eq("glm-asr-nano-2512") || m.eq("fun-asr-nano-2512") => {
                serde_json::json!({})
            }
            _ => {
                serde_json::json!({})
            }
        };

        context
            .request
            .set_body(serde_json::to_vec(&result).unwrap().into());

        Ok(Default::default())
    }
}

// 导出插件
export!(AhaModelRequestWrapperPlugin);
