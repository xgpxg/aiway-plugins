use aiway_plugin::http::{request, response};
use aiway_plugin::PluginContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export_wasm,
};

/// Echo插件
///
/// 该插件无实际功能，仅输出context
pub struct EchoPlugin;

impl EchoPlugin {
    pub fn new() -> Self {
        Self {}
    }
}

#[async_trait]
impl Plugin for EchoPlugin {
    fn name(&self) -> &str {
        "Echo"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: Default::default(),
            description: "原样输出参数，无实际功能".to_string(),
            readme: None,
        }
    }

    // 实现插件逻辑
    async fn on_request(
        &self,
        _config: &Value,
        head: &mut request::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        ctx.log_info(&format!("[{}] {} {}", ctx.request_id(), head.method, head.uri));
        Ok(())
    }

    async fn on_response(
        &self,
        _config: &Value,
        head: &mut response::Parts,
        ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        ctx.log_info(&format!("[{}] {} {}", ctx.request_id(), head.status, head.status.as_u16()));
        Ok(())
    }
}

// 导出 WASM 插件
export_wasm!(EchoPlugin);
