use aiway_plugin::protocol::context::http::{self, HeaderName, HeaderValue};
use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::{json, Value};
use aiway_plugin::{
    async_trait, export, plugin_version, serde_json, Bytes, Plugin, PluginError, PluginInfo,
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
    fn name(&self) -> &'static str {
        "zhipu"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "智谱AI模型适配（请求/响应转换）".to_string(),
        }
    }

    // ========== 请求体处理（原 mi） ==========
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

    // ========== 响应头处理（原 mo） ==========
    async fn on_response(
        &self,
        _config: &Value,
        head: &mut http::response::Parts,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        if let Some(provider) = ctx.get_proxy_model_provider() {
            if !provider.api_url.is_empty() {
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
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        let provider = ctx.get_proxy_model_provider();

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

    /// 上传文件
    fn upload_file(&self, file: &str, api_key: &str) -> Result<String, PluginError> {
        let client = reqwest::blocking::Client::new();

        let form_data = if file.starts_with("http://") || file.starts_with("https://") {
            let file_response = client.get(file).send().map_err(|e| {
                PluginError::ExecuteError(format!("Failed to download file: {}", e))
            })?;

            let file_content = file_response.bytes().map_err(|e| {
                PluginError::ExecuteError(format!("Failed to read file content: {}", e))
            })?;

            let mut form = reqwest::blocking::multipart::Form::new();
            form = form.part(
                "file",
                reqwest::blocking::multipart::Part::bytes(file_content.to_vec())
                    .file_name("uploaded_file.wav")
                    .mime_str("application/octet-stream")
                    .map_err(|e| PluginError::ExecuteError(format!("Invalid MIME type: {}", e)))?,
            );
            form
        } else {
            let decoded_bytes = STANDARD.decode(&file).map_err(|e| {
                PluginError::ExecuteError(format!("Failed to decode base64: {}", e))
            })?;

            let mut form = reqwest::blocking::multipart::Form::new();
            form = form.part(
                "file",
                reqwest::blocking::multipart::Part::bytes(decoded_bytes)
                    .file_name("decoded_file.wav")
                    .mime_str("application/octet-stream")
                    .map_err(|e| PluginError::ExecuteError(format!("Invalid MIME type: {}", e)))?,
            );
            form
        };

        let form_data = form_data.text("purpose", "voice-clone-input");

        let response = client
            .post(Self::UPLOAD_FILE_URL)
            .header("Authorization", format!("Bearer {}", api_key))
            .multipart(form_data)
            .send()
            .map_err(|e| PluginError::ExecuteError(format!("Request failed: {}", e)))?;

        let status = response.status();
        let response_text = response
            .text()
            .map_err(|e| PluginError::ExecuteError(format!("Failed to read response: {}", e)))?;

        if !status.is_success() {
            return Err(PluginError::ExecuteError(format!(
                "Upload failed with status {}: {}",
                status, response_text
            )));
        }

        println!("Upload successful: {}", response_text);

        let response_json: Value = serde_json::from_str(&response_text).map_err(|e| {
            PluginError::ExecuteError(format!("Failed to parse response JSON: {}", e))
        })?;

        let file_id = response_json["id"]
            .as_str()
            .ok_or(PluginError::ExecuteError(
                "File ID not found in response".to_string(),
            ))?
            .to_string();

        Ok(file_id)
    }

    fn download_file(&self, file_id: &str, api_key: &str) -> Result<Vec<u8>, PluginError> {
        let url = Self::DOWNLOAD_FILE_URL.replace("<file_id>", file_id);

        let client = reqwest::blocking::Client::new();

        let response = client
            .get(&url)
            .header("Authorization", format!("Bearer {}", api_key))
            .send()
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        let bytes = response
            .bytes()
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        Ok(bytes.to_vec())
    }
}
// 导出插件
export!(ZhiPuPlugin);
