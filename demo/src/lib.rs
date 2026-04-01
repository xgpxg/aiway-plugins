use aiway_plugin::protocol::context::http::request;
use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    async_trait, export, plugin_version, Plugin, PluginError, PluginInfo, Version,
};

// 示例插件
pub struct DemoPlugin;

impl DemoPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for DemoPlugin {
    fn name(&self) -> &'static str {
        "demo"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "Demo Plugin".to_string(),
        }
    }

    async fn on_request(
        &self,
        _config: &Value,
        _head: &mut request::Parts,
        _ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        // 这里实现插件逻辑
        Ok(())
    }
}

// 导出插件
export!(DemoPlugin);
