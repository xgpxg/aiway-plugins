use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version, serde_json,
};

/// 智谱AI模型输入适配插件
pub struct ZhiPuMO;

impl ZhiPuMO {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for ZhiPuMO {
    fn name(&self) -> &'static str {
        "zhipu-mo"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "智谱AI模型输出适配".to_string(),
        }
    }

    // 实现插件逻辑
    async fn execute(&self, context: &HttpContext, _config: &Value) -> Result<Value, PluginError> {
        if !context.response.is_success() {
            return Ok(Default::default());
        }

        let provider = context.inner_state.get_model_provider();

        if provider.is_none() {
            return Err(PluginError::ExecuteError(
                "provider is not found".to_string(),
            ));
        }
        let provider = provider.unwrap();

        let body = &serde_json::from_slice::<Value>(context.response.get_body().unwrap())
            .map_err(|e| PluginError::ExecuteError(e.to_string()))?;

        match provider.api_url.as_str() {
            // 音色克隆
            p if !p.is_empty() => {
                let file_id = body["file_id"].as_str().ok_or(PluginError::ExecuteError(
                    "file_id field is not found".to_string(),
                ))?;

                let bytes = self.download_file(
                    file_id,
                    &provider
                        .api_key
                        .ok_or("api_key is not found")
                        .map_err(|e| PluginError::ExecuteError(e.to_string()))?,
                )?;

                context.response.set_body(bytes.into());
                context.response.insert_header("Content-Type", "audio/wav");
            }
            _ => {}
        }

        Ok(Default::default())
    }
}
impl ZhiPuMO {
    const DOWNLOAD_FILE_URL: &'static str =
        "https://open.bigmodel.cn/api/paas/v4/files/<file_id>/content";

    /// 智谱响应示例：
    /// ```json
    /// {
    ///     "file_id": "1768118482370-806700eb9d324ef89bae53574e1afd0c.wav",
    ///     "file_purpose": "voice-clone-output",
    ///     "request_id": "20260111160114b217be77f6904173",
    ///     "voice": "3af4deae-494f-5af1-82a9-0643d4dba0ad"
    /// }
    /// ```
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
export!(ZhiPuMO);
