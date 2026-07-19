use aiway_plugin::http::{response, HeaderName, HeaderValue};
use aiway_plugin::PluginContext;
use aiway_plugin::serde_json::{json, Value};
use aiway_plugin::{
    async_trait, export_wasm, serde_json, Bytes, Plugin, PluginError, PluginInfo,
    Version,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// Aha 模型适配插件
pub struct AhaPlugin;

impl AhaPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for AhaPlugin {
    fn name(&self) -> &str {
        "aha"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "Aha 模型适配（请求/响应转换）".to_string(),
            readme: None,
        }
    }

    // ========== 请求体处理（原 mi） ==========
    async fn on_request_body(
        &self,
        _config: &Value,
        body: &mut Option<Bytes>,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let body_val = &serde_json::from_slice::<Value>(body.as_ref().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;
        let model = body_val["model"]
            .as_str()
            .ok_or(PluginError::ExecuteError(
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
                body_val.clone()
            }
            m if m.eq("rmbg2.0") => {
                let image = body_val["image"].as_str().ok_or(PluginError::ExecuteError(
                    "input field is not found".to_string(),
                ))?;
                json!({
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
                let input = body_val["input"].as_str().ok_or(PluginError::ExecuteError(
                    "input field is not found".to_string(),
                ))?;
                let voice = body_val["voice"].as_str().ok_or(PluginError::ExecuteError(
                    "voice field is not found".to_string(),
                ))?;
                let voice_text = body_val["voice_text"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "voice_text field is not found".to_string(),
                    ))?;
                json!({
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
                json!({})
            }
            _ => {
                json!({})
            }
        };

        *body = Some(
            serde_json::to_vec(&result)
                .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                .into(),
        );

        Ok(())
    }

    // ========== 响应头处理（原 mo） ==========
    async fn on_response(
        &self,
        _config: &Value,
        head: &mut response::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        if let Some(model_name) = ctx.get_model_name() {
            if model_name.starts_with("voxcpm") {
                head.headers.insert(
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("audio/wav"),
                );
            }
        }
        Ok(())
    }

    // ========== 响应体处理（原 mo） ==========
    fn on_response_body(
        &self,
        _config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let model_name = if let Some(name) = ctx.get_model_name() {
            name
        } else {
            return Ok(());
        };

        let body_val = &serde_json::from_slice::<Value>(body.as_ref().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        match model_name.as_str() {
            m if m.eq("rmbg2.0") => {
                let image_base64 = body_val["choices"][0]["message"]["content"][0]["image_url"]["url"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "image_url field is not found".to_string(),
                    ))?;
                let response = json!({
                    "data": [{
                        "b64_json": image_base64,
                        "revised_prompt": null
                    }]
                });
                *body = Some(
                    serde_json::to_vec(&response)
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                        .into(),
                );
            }
            m if m.starts_with("voxcpm") => {
                let base64 = body_val["choices"][0]["message"]["content"][0]["audio_url"]["url"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "audio_url field is not found".to_string(),
                    ))?;

                let base64 = base64.strip_prefix("data:audio/wav;base64,").unwrap_or(base64);

                let bytes = STANDARD
                    .decode(base64)
                    .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

                *body = Some(bytes.into());
            }
            m if m.eq("glm-asr-nano-2512") || m.eq("fun-asr-nano-2512") => {}
            _ => {}
        };

        Ok(())
    }
}

// 导出 WASM 插件
export_wasm!(AhaPlugin);
