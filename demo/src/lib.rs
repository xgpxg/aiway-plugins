use aiway_plugin::http::request;
use aiway_plugin::PluginContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export_wasm,
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
    fn name(&self) -> &str {
        "demo"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "Demo Plugin".to_string(),
        }
    }

    async fn on_request(
        &self,
        _config: &Value,
        _head: &mut request::Parts,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        Ok(())
    }
}

// 导出 WASM 插件
export_wasm!(DemoPlugin);
