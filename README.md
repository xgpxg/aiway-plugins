# 已支持的插件

| 插件                  | 下载                                                                                                                                                                                                                                  | 当前版本  | 类型        | 功能                                                      |
|---------------------|-------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------|-----------|---------------------------------------------------------|
| **demo**            | [x86_64](https://package-release.coderbox.cn/aiway/plugins/demo/0.1.1/x86_64/libdemo.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/demo/0.1.1/aarch64/libdemo.so)                                             | 0.1.1 | 示例插件      | Demo Plugin，无实际功能                                       |
| **echo**            | [x86_64](https://package-release.coderbox.cn/aiway/plugins/echo/0.1.1/x86_64/libecho.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/echo/0.1.1/aarch64/libecho.so)                                             | 0.1.1 | 调试插件      | 原样输出参数，打印请求和响应上下文                                       |
| **rewrite-path**    | [x86_64](https://package-release.coderbox.cn/aiway/plugins/rewrite-path/0.1.1/x86_64/librewrite_path.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/rewrite-path/0.1.1/aarch64/librewrite_path.so)             | 0.1.1 | 路径重写      | 重写网关收到的路径，支持正则匹配                                        |
| **header-operator** | [x86_64](https://package-release.coderbox.cn/aiway/plugins/header-operator/0.1.1/x86_64/libheader_operator.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/header-operator/0.1.1/aarch64/libheader_operator.so) | 0.1.1 | Header 操作 | 添加、删除和修改 HTTP 请求头/响应头                                   |
| **aha**             | [x86_64](https://package-release.coderbox.cn/aiway/plugins/aha-mi/0.1.1/x86_64/libaha.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/aha-mi/0.1.1/aarch64/libaha.so)                                           | 0.1.1 | 模型转换      | Aha 模型请求参数转换（minicpm4、qwen2.5vl、qwen3、rmbg2.0、voxcpm 等） |
| **bailian**         | [x86_64](https://package-release.coderbox.cn/aiway/plugins/bailian-mi/0.1.1/x86_64/libbailia.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/bailian-mi/0.1.1/aarch64/libbailian.so)                            | 0.1.1 | 模型转换      | 阿里百炼平台模型接口请求适配（文生图）                                     |
| **volcengine**      | [x86_64](https://package-release.coderbox.cn/aiway/plugins/volcengine-mi/0.1.1/x86_64/libvolcengine.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/volcengine-mi/0.1.1/aarch64/libvolcengine.so)               | 0.1.1 | 模型转换      | 火山引擎模型接口请求适配                                            |
| **zhipu**           | [x86_64](https://package-release.coderbox.cn/aiway/plugins/zhipu-mi/0.1.1/x86_64/libzhipu.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/zhipu-mi/0.1.1/aarch64/libzhipu.so)                                   | 0.1.1 | 模型转换      | 智谱 AI 模型输入适配（音色克隆、文件上传）                                 |

> - 所有插件为 cdylib 动态库
> - 要求：为实现更好的兼容性，构建插件的 glibc 版本须 <= 2.28

## 自定义插件

可参考 [示例插件](/demo) 实现自定义插件。

[插件文档](https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md)

## 编译插件

```shell
cargo build -r --worksapce
```