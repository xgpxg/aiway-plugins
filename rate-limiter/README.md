# rate-limiter

请求限流插件，基于滑动窗口算法，支持按路径、客户端 IP 或自定义 Header 进行限流。

## 配置

| 字段              | 类型     | 默认值                    | 说明                                  |
|-----------------|--------|------------------------|-------------------------------------|
| `strategy`      | string | `"ip"`                 | 限流策略：`"path"` / `"ip"` / `"header"` |
| `max_requests`  | number | `100`                  | 时间窗口内最大请求数                          |
| `window`        | number | `60`                   | 时间窗口（秒）                             |
| `header_name`   | string | —                      | Header 策略时指定的 header 名称             |
| `ip_header`     | string | `"X-Forwarded-For"`   | 客户端 IP 来源 header                    |
| `status_code`   | number | `429`                  | 限流响应状态码                             |
| `response_body` | string | `"Too Many Requests"` | 限流响应体                               |

## 示例

**按路径限流，每个路径 60 秒内最多 100 次**：

```json
{
  "strategy": "path",
  "max_requests": 100,
  "window": 60
}
```

**按客户端 IP 限流，每个 IP 10 秒内最多 5 次**：

```json
{
  "strategy": "ip",
  "max_requests": 5,
  "window": 10
}
```

**按自定义 Header 限流**：

```json
{
  "strategy": "header",
  "header_name": "X-Api-Key",
  "max_requests": 50,
  "window": 60
}
```

## 注意

- 时间戳由宿主侧提供（`request_ts`），不依赖 WASM 内部时钟
- WASM 插件实例池化运行，每个实例维护独立的限流状态。若池中存在 N 个实例，实际限流阈值为 `max_requests × N`。需要精确限流时，建议将实例池大小设为 1
