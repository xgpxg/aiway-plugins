use aiway_plugin::http::{self, HeaderName, HeaderValue};
use aiway_plugin::{FormPart, HttpRequestBuilder, PluginContext};
use aiway_plugin::serde_json::{json, Value};
use aiway_plugin::{
    async_trait, export_wasm, serde_json, Bytes, Plugin, PluginError, PluginInfo,
    Version,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// 智谱AI模型适配插件
pub struct ZhiPuPlugin;

impl ZhiPuPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for ZhiPuPlugin {
    fn name(&self) -> &str {
        "zhipu"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "智谱AI模型适配（请求/响应转换）".to_string(),
            readme: None,
        }
    }

    // ========== 请求体处理 ==========
    async fn on_request_body(
        &self,
        _config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let provider = ctx.get_model_provider();

        if provider.is_none() {
            return Err(PluginError::ExecuteError(
                "provider is not found".to_string(),
            ));
        }
        let provider = provider.unwrap();

        let body_val = serde_json::from_slice::<Value>(body.as_ref().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        match provider.api_url.as_str() {
            // 音色克隆
            p if p.ends_with("/voice/clone") => {
                let voice = body_val["voice"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "voice field is not found".to_string(),
                    ))?;

                // 上传文件得到文件ID
                let file_id = self.upload_file(
                    voice,
                    &provider
                        .api_key
                        .ok_or("api_key is not found")
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?,
                    ctx,
                )?;

                let model = body_val["model"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "model field is not found".to_string(),
                    ))?;
                let input = body_val["input"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "input field is not found".to_string(),
                    ))?;
                let result = json!({
                    "model": model,
                    "voice_name": uuid::Uuid::new_v4().to_string(),
                    "input": input,
                    "file_id": file_id,
                });
                *body = Some(
                    serde_json::to_vec(&result)
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                        .into(),
                )
            }
            _ => {
                *body = Some(
                    serde_json::to_vec(&body_val)
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                        .into(),
                )
            }
        }

        Ok(())
    }

    // ========== 响应头处理 ==========
    async fn on_response(
        &self,
        _config: &Value,
        head: &mut http::response::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        if let Some(provider) = ctx.get_model_provider() {
            if !provider.api_url.is_empty() {
                head.headers.insert(
                    HeaderName::from_static("content-type"),
                    HeaderValue::from_static("audio/wav"),
                );
            }
        }
        Ok(())
    }

    // ========== 响应体处理 ==========
    async fn on_response_body(
        &self,
        _config: &Value,
        body: &mut Option<Bytes>,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let provider = ctx.get_model_provider();

        if provider.is_none() {
            return Err(PluginError::ExecuteError(
                "provider is not found".to_string(),
            ));
        }
        let provider = provider.unwrap();

        let body_val = &serde_json::from_slice::<Value>(body.as_ref().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        match provider.api_url.as_str() {
            // 音色克隆
            p if !p.is_empty() => {
                let file_id = body_val["file_id"]
                    .as_str()
                    .ok_or(PluginError::ExecuteError(
                        "file_id field is not found".to_string(),
                    ))?;

                let bytes = self.download_file(
                    file_id,
                    &provider
                        .api_key
                        .ok_or("api_key is not found")
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?,
                    ctx,
                )?;

                *body = Some(bytes.into());
            }
            _ => {}
        }

        Ok(())
    }
}
impl ZhiPuPlugin {
    const UPLOAD_FILE_URL: &'static str = "https://open.bigmodel.cn/api/paas/v4/files";
    const DOWNLOAD_FILE_URL: &'static str =
        "https://open.bigmodel.cn/api/paas/v4/files/<file_id>/content";

    fn upload_file(&self, file: &str, api_key: &str, ctx: &dyn PluginContext) -> Result<String, PluginError> {
        let file_content = if file.starts_with("http://") || file.starts_with("https://") {
            let req = HttpRequestBuilder::new("GET", file)
                .timeout_ms(30_000)
                .build();
            let resp = ctx.http_request(&req)
                .map_err(|e| PluginError::ExecuteError(format!("Failed to download file: {}", e)))?;
            if resp.status != 200 {
                return Err(PluginError::ExecuteError(format!(
                    "Download failed with status {}", resp.status
                )));
            }
            resp.body
        } else {
            STANDARD.decode(file).map_err(|e| {
                PluginError::ExecuteError(format!("Failed to decode base64: {}", e))
            })?
        };

        // 使用宿主侧 multipart API
        let req = HttpRequestBuilder::new("POST", Self::UPLOAD_FILE_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .add_multipart_part(FormPart {
                key: "file".to_string(),
                value: file_content,
                file_name: Some("uploaded_file.wav".to_string()),
                mime_type: Some("application/octet-stream".to_string()),
            })
            .add_multipart_part(FormPart {
                key: "purpose".to_string(),
                value: b"voice-clone-input".to_vec(),
                file_name: None,
                mime_type: None,
            })
            .timeout_ms(60_000)
            .build();

        let resp = ctx.http_request(&req)
            .map_err(|e| PluginError::ExecuteError(format!("Upload request failed: {}", e)))?;

        if resp.status != 200 {
            let text = String::from_utf8_lossy(&resp.body);
            return Err(PluginError::ExecuteError(format!(
                "Upload failed with status {}: {}", resp.status, text
            )));
        }

        let response_json: Value = serde_json::from_slice(&resp.body)
            .map_err(|e| PluginError::ExecuteError(format!("Failed to parse response JSON: {}", e)))?;

        let file_id = response_json["id"]
            .as_str()
            .ok_or(PluginError::ExecuteError("File ID not found in response".to_string()))?
            .to_string();

        Ok(file_id)
    }

    fn download_file(&self, file_id: &str, api_key: &str, ctx: &dyn PluginContext) -> Result<Vec<u8>, PluginError> {
        let url = Self::DOWNLOAD_FILE_URL.replace("<file_id>", file_id);

        let req = HttpRequestBuilder::new("GET", url)
            .header("Authorization", format!("Bearer {}", api_key))
            .timeout_ms(30_000)
            .build();

        let resp = ctx.http_request(&req)
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        Ok(resp.body)
    }
}
// 导出 WASM 插件
export_wasm!(ZhiPuPlugin);
