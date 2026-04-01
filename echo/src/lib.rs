use aiway_plugin::protocol::context::http::{request, response};
use aiway_plugin::protocol::context::HttpContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{
    Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version,
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
    fn name(&self) -> &'static str {
        "Echo"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: plugin_version!(),
            default_config: Default::default(),
            description: "原样输出参数，无实际功能".to_string(),
        }
    }

    // 实现插件逻辑
    async fn on_request(
        &self,
        _config: &Value,
        head: &mut request::Parts,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        println!("========== 请求信息 ==========");
        println!("URI: {}", head.uri);
        println!("方法：{}", head.method);

        // 打印请求头
        println!("\n--- 请求头 ---");
        for (key, value) in &head.headers {
            println!("{}: {:?}", key, value);
        }
        
        // 尝试从 context 中获取更多信息
        println!("\n--- Context 信息 ---");
        println!("请求 ID: {:?}", ctx.request_id());
        println!("上下文：{:?}", ctx);
        
        println!("============================\n");
        Ok(Default::default())
    }

    async fn on_response(
        &self,
        _config: &Value,
        head: &mut response::Parts,
        ctx: &mut HttpContext,
    ) -> Result<(), PluginError> {
        println!("========== 响应信息 ==========");
        println!("状态码：{}", head.status);
        println!("版本：{:?}", head.version);
        
        // 打印响应头
        println!("\n--- 响应头 ---");
        for (key, value) in &head.headers {
            println!("{}: {:?}", key, value);
        }

        // 尝试从 context 中获取更多信息
        println!("\n--- Context 信息 ---");
        println!("请求 ID: {:?}", ctx.request_id());
        println!("上下文：{:?}", ctx);
        
        println!("============================\n");
        Ok(Default::default())
    }
}

// 导出插件
export!(EchoPlugin);
