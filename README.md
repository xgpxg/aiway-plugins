# 已支持的插件

| 插件                  | 下载                                                                                                 | 当前版本  | 类型        | 功能                                                      |
|---------------------|----------------------------------------------------------------------------------------------------|-------|-----------|---------------------------------------------------------|
| **demo**            | [下载](https://package-release.coderbox.cn/aiway/plugins/demo/0.1.2/demo.wasm)                       | 0.1.2 | 示例插件      | Demo Plugin，无实际功能                                       |
| **echo**            | [下载](https://package-release.coderbox.cn/aiway/plugins/echo/0.1.2/echo.wasm)                       | 0.1.2 | 调试插件      | 原样输出参数，打印请求和响应上下文                                       |
| **rewrite-path**    | [下载](https://package-release.coderbox.cn/aiway/plugins/rewrite-path/0.1.2/rewrite_path.wasm)       | 0.1.2 | 路径重写      | 重写网关收到的路径，支持正则匹配                                        |
| **header-operator** | [下载](https://package-release.coderbox.cn/aiway/plugins/header-operator/0.1.2/header_operator.wasm) | 0.1.2 | Header 操作 | 添加、删除和修改 HTTP 请求头/响应头                                   |
| **aha**             | [下载](https://package-release.coderbox.cn/aiway/plugins/aha/0.1.2/aha.wasm)                         | 0.1.2 | 模型转换      | Aha 模型请求参数转换（minicpm4、qwen2.5vl、qwen3、rmbg2.0、voxcpm 等） |
| **bailian**         | [下载](https://package-release.coderbox.cn/aiway/plugins/bailian/0.1.2/bailia.wasm)                  | 0.1.2 | 模型转换      | 阿里百炼平台模型接口请求适配（文生图）                                     |
| **volcengine**      | [下载](https://package-release.coderbox.cn/aiway/plugins/volcengine/0.1.2/volcengine.wasm)           | 0.1.2 | 模型转换      | 火山引擎模型接口请求适配                                            |
| **zhipu**           | [下载](https://package-release.coderbox.cn/aiway/plugins/zhipu/0.1.2/zhipu.wasm)                     | 0.1.2 | 模型转换      | 智谱 AI 模型输入适配（音色克隆、文件上传）                                 |
| **rate-limiter**    | [下载](https://package-release.coderbox.cn/aiway/plugins/rate-limiter/0.1.0/rate_limiter.wasm)       | 0.1.0 | 通用        | 请求限流                                                    |

## 自定义插件

可参考 [示例插件](/demo) 实现自定义插件。

[插件文档](https://xgpxg.github.io/aiway/plugins/overview.html)

## 编译插件

```shell
cargo build -r --workspace --target wasm32-wasip1
```