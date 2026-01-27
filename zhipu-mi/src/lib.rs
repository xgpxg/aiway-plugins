use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::{Value, json};
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version, serde_json,
};
use base64::Engine;
use base64::engine::general_purpose::STANDARD;

/// 智谱AI模型输入适配插件
pub struct ZhiPuMI;

impl ZhiPuMI {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for ZhiPuMI {
    fn name(&self) -> &'static str {
        "zhipu-mi"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "智谱AI模型输入适配".to_string(),
        }
    }

    // 实现插件逻辑
    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        let provider = context.inner_state.get_model_provider();

        if provider.is_none() {
            return Err(PluginError::ExecuteError(
                "provider is not found".to_string(),
            ));
        }
        let provider = provider.unwrap();

        let body = serde_json::from_slice::<Value>(context.request.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        match provider.api_url.as_str() {
            // 音色克隆
            p if p.ends_with("/voice/clone") => {
                let voice = body["voice"].as_str().ok_or(PluginError::ExecuteError(
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

                let model = body["model"].as_str().ok_or(PluginError::ExecuteError(
                    "model field is not found".to_string(),
                ))?;
                let input = body["input"].as_str().ok_or(PluginError::ExecuteError(
                    "input field is not found".to_string(),
                ))?;
                let result = json!({
                    "model": model,
                    "voice_name": uuid::Uuid::new_v4().to_string(),
                    "input": input,
                    "file_id": file_id,
                });
                context.request.set_body(
                    serde_json::to_vec(&result)
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                        .into(),
                )
            }
            _ => context.request.set_body(
                serde_json::to_vec(&body)
                    .map_err(|e| PluginError::ExecuteError(e.to_string()))?
                    .into(),
            ),
        }

        Ok(Default::default())
    }
}
impl ZhiPuMI {
    const UPLOAD_FILE_URL: &'static str = "https://open.bigmodel.cn/api/paas/v4/files";

    ///  上传文件
    /// - file：可能是网络地址或base64编码格式
    /// - api_key：智谱AI的API密钥
    fn upload_file(&self, file: &str, api_key: &str) -> Result<String, PluginError> {
        let client = reqwest::blocking::Client::new();

        // 判断文件是URL还是base64编码
        let form_data = if file.starts_with("http://") || file.starts_with("https://") {
            // 如果是URL，则下载文件内容
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
            // 如果是base64编码，先解码
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

        // 添加其他必需的字段
        let form_data = form_data.text("purpose", "voice-clone-input");

        // 发送请求
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

        // 解析响应获取文件ID
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
}
// 导出插件
export!(ZhiPuMI);
