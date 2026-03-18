# 已支持的插件

| 插件                  | 下载                                                                                                                                                                                                                                 | 当前版本  | 类型        | 功能                                                      |
|---------------------|------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------|-------|-----------|---------------------------------------------------------|
| **demo**            | [x86_64](https://package-release.coderbox.cn/aiway/plugins/demo/0.1.0/x86_64/libdemo.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/demo/0.1.0/aarch64/libdemo.so)                                             | 0.1.0 | 示例插件      | Demo Plugin，无实际功能                                       |
| **echo**            | [x86_64](https://package-release.coderbox.cn/aiway/plugins/echo/0.1.0/x86_64/libecho.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/echo/0.1.0/aarch64/libecho.so)                                             | 0.1.0 | 调试插件      | 原样输出参数，打印请求和响应上下文                                       |
| **rewrite-path**    | [x86_64](https://package-release.coderbox.cn/aiway/plugins/rewrite-path/0.1.0/x86_64/librewrite_path.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/rewrite-path/0.1.0/aarch64/librewrite_path.so)             | 0.1.0 | 路径重写      | 重写网关收到的路径，支持正则匹配                                        |
| **aha-mi**          | [x86_64](https://package-release.coderbox.cn/aiway/plugins/aha-mi/0.1.0/x86_64/libaha_mi.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/aha-mi/0.1.0/aarch64/libaha_mi.so)                                     | 0.1.0 | 模型输入      | Aha 模型请求参数转换（minicpm4、qwen2.5vl、qwen3、rmbg2.0、voxcpm 等） |
| **aha-mo**          | [x86_64](https://package-release.coderbox.cn/aiway/plugins/aha-mo/0.1.0/x86_64/libaha_mo.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/aha-mo/0.1.0/aarch64/libaha_mo.so)                                     | 0.1.0 | 模型输出      | Aha 模型响应参数转换（图像、音频处理）                                   |
| **bailian-mi**      | [x86_64](https://package-release.coderbox.cn/aiway/plugins/bailian-mi/0.1.0/x86_64/libbailian_mi.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/bailian-mi/0.1.0/aarch64/libbailian_mi.so)                     | 0.1.0 | 模型输入      | 阿里百炼平台模型接口请求适配（文生图）                                     |
| **bailian-mo**      | [x86_64](https://package-release.coderbox.cn/aiway/plugins/bailian-mo/0.1.0/x86_64/libbailian_mo.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/bailian-mo/0.1.0/aarch64/libbailian_mo.so)                     | 0.1.0 | 模型输出      | 阿里百炼平台模型接口响应适配（文生图）                                     |
| **header-operator** | [x86_64](https://package-release.coderbox.cn/aiway/plugins/header-operator/0.1.0/x86_64/libheader_operator.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/header-operator/0.1.0/aarch64/libheader_operator.so) | 0.1.0 | Header 操作 | 添加、删除和修改 HTTP 请求头/响应头                                   |
| **volcengine-mi**   | [x86_64](https://package-release.coderbox.cn/aiway/plugins/volcengine-mi/0.1.0/x86_64/libvolcengine_mi.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/volcengine-mi/0.1.0/aarch64/libvolcengine_mi.so)         | 0.1.0 | 模型输入      | 火山引擎模型接口请求适配                                            |
| **volcengine-mo**   | [x86_64](https://package-release.coderbox.cn/aiway/plugins/volcengine-mo/0.1.0/x86_64/libvolcengine_mo.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/volcengine-mo/0.1.0/aarch64/libvolcengine_mo.so)         | 0.1.0 | 模型输出      | 火山引擎模型接口响应适配                                            |
| **zhipu-mi**        | [x86_64](https://package-release.coderbox.cn/aiway/plugins/zhipu-mi/0.1.0/x86_64/libzhipu_mi.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/zhipu-mi/0.1.0/aarch64/libzhipu_mi.so)                             | 0.1.0 | 模型输入      | 智谱 AI 模型输入适配（音色克隆、文件上传）                                 |
| **zhipu-mo**        | [x86_64](https://package-release.coderbox.cn/aiway/plugins/zhipu-mo/0.1.0/x86_64/libzhipu_mo.so)  /  [aarch64](https://package-release.coderbox.cn/aiway/plugins/zhipu-mo/0.1.0/aarch64/libzhipu_mo.so)                             | 0.1.0 | 模型输出      | 智谱 AI 模型输出适配（音频文件下载）                                    |

> - 带 `-mi` 后缀：Model Input，负责模型请求参数的转换和适配
> - 带 `-mo` 后缀：Model Output，负责模型响应结果的转换和适配
> - 所有插件为 cdylib 动态库
> - 要求：为实现更好的兼容性，构建插件的 glibc 版本须 <= 2.28

## 自定义插件

可参考 [示例插件](/demo) 实现自定义插件。

[插件文档](https://aiway.coderbox.cn/doc.html?path=docs/plugins/introduction.md)