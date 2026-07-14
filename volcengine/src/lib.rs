use aiway_plugin::protocol::context::http::response;
use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    async_trait, export, plugin_version, Bytes, Plugin, PluginError, PluginInfo, Version,
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
    fn name(&self) -> &'static str {
        "volcengine"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "Volcengine 模型适配（请求/响应转换）".to_string(),
        }
    }

    async fn on_request_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    async fn on_response(
        &self,
        _config: &Value,
        _head: &mut response::Parts,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }

    fn on_response_body(
        &self,
        _config: &Value,
        _body: &mut Option<Bytes>,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

// 导出插件
export!(VolcenginePlugin);
