use aiway_plugin::protocol::gateway::HttpContext;
use aiway_plugin::serde_json::Value;
use aiway_plugin::{Plugin, PluginError, PluginInfo, Version, async_trait, export, plugin_version};

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
    async fn execute(&self, context: &HttpContext, config: &Value) -> Result<Value, PluginError> {
        // 输出请求上下文
        println!("=== Request Context ===");
        println!("Request ID: {}", context.request.request_id);
        println!(
            "Method: {}",
            context.request.get_method().unwrap_or("UNKNOWN")
        );
        println!("Path: {}", context.request.get_path());
        println!("Host: {}", context.request.host);

        // 输出请求头
        println!("Headers:");
        for header in &context.request.headers {
            println!("  {}: {}", header.key(), header.value());
        }

        // 输出查询参数
        println!("Query Params:");
        for param in &context.request.query {
            println!("  {}: {}", param.key(), param.value());
        }

        // 输出请求体
        if let Some(body) = context.request.get_body() {
            match std::str::from_utf8(body) {
                Ok(body_str) => {
                    println!("Body (text): {}", body_str);
                }
                Err(_) => {
                    println!("Body (binary): {} bytes", body.len());
                }
            }
        } else {
            println!("Body: None");
        }

        // 输出响应上下文
        println!("\n=== Response Context ===");
        println!("Status: {:?}", context.response.get_status());

        // 输出响应头
        println!("Headers:");
        for header in &context.response.headers {
            println!("  {}: {}", header.key(), header.value());
        }

        // 输出响应体
        if let Some(body) = context.response.get_body() {
            match std::str::from_utf8(body) {
                Ok(body_str) => {
                    println!("Body (text): {}", body_str);
                }
                Err(_) => {
                    println!("Body (binary): {} bytes", body.len());
                }
            }
        } else {
            println!("Body: None");
        }

        if let Some(stream_body) = context.response.stream_body.get() {
            if let Some(_) = stream_body.as_ref() {
                println!("Stream Body: Stream<Vec<u8>>");
            } else {
                println!("Stream Body: None");
            }
        }

        println!("\n=== Plugin Config ===");

        println!("Config: {:?}", config);

        println!("\n=== End Context ===");

        Ok(Default::default())
    }
}

// 导出插件
export!(EchoPlugin);
