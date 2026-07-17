use aiway_plugin::http::response;
use aiway_plugin::protocol::context::PluginContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    async_trait, export_wasm, Bytes, Plugin, PluginError, PluginInfo, Version,
};

/// Volcengine 模型适配插件
pub struct VolcenginePlugin;

impl VolcenginePlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for VolcenginePlugin {
    fn name(&self) -> &str {
        "volcengine"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "Volcengine 模型适配（请求/响应转换）".to_string(),
        }
    }

    async fn on_request_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn on_response(
        &self,
        _config: &Value,
        _head: &mut response::Parts,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_response_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

// 导出 WASM 插件
export_wasm!(VolcenginePlugin);
