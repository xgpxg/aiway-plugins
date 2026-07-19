# jwt-validator

JWT Token 验证插件。支持解码、HMAC 验签、Claims 校验和 Claims 注入请求头。

## 配置

| 字段                    | 类型       | 默认值                       | 说明                                                               |
|-----------------------|----------|---------------------------|------------------------------------------------------------------|
| `token_source.from`   | string   | `"header"`                | Token 来源：`"header"` 或 `"query"`                                  |
| `token_source.name`   | string   | `"Authorization"`         | Header 名称或 Query 参数名                                             |
| `token_source.prefix` | string   | `"Bearer "`               | Token 前缀（仅 header 模式）                                            |
| `secret`              | string   | —                         | HMAC 密钥。不配置则跳过验签，仅解码                                             |
| `algorithm`           | string   | `"HS256"`                 | 签名算法：`HS256` / `HS384` / `HS512`                                 |
| `validate.iss`        | string   | —                         | 期望的签发者（iss）                                                      |
| `validate.aud`        | string[] | —                         | 允许的受众列表（aud）                                                     |
| `validate.leeway`     | number   | `60`                      | exp/nbf 校验的时钟偏差（秒）                                               |
| `claims_to_headers`   | object   | `{"sub":"X-JWT-Subject"}` | 将 claims 映射为请求头，如 `{"sub":"X-JWT-Subject", "name":"X-JWT-Name"}` |
| `strip_token`         | bool     | `false`                   | 校验后移除 Authorization header                                       |

## 示例

**仅解码提取 sub（无需验签）**：

```json
{
  "claims_to_headers": {
    "sub": "X-JWT-Subject"
  }
}
```

**HS256 验签 + Claims 校验**：

```json
{
  "secret": "my-hmac-secret",
  "validate": {
    "iss": "https://auth.example.com",
    "aud": [
      "api-gateway"
    ]
  },
  "claims_to_headers": {
    "sub": "X-JWT-Subject",
    "iss": "X-JWT-Issuer"
  },
  "strip_token": true
}
```

## 注意

- 仅支持对称算法 HS256/HS384/HS512。RS256/ES256 等非对称算法只解码不做验签
- `exp`、`nbf` 使用系统时钟，`leeway` 可缓解时钟偏差问题
