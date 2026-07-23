use aiway_plugin::http::{request, response, HeaderName, HeaderValue};
use aiway_plugin::PluginContext;
use aiway_plugin::serde_json::{self, json, Value};
use aiway_plugin::{
    async_trait, export_wasm, Plugin, PluginError, PluginInfo, Version,
};
use base64::Engine as _;
use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::time::{SystemTime, UNIX_EPOCH};

/// JWT 验证与 Claims 提取插件
pub struct JwtValidator;

impl JwtValidator {
    pub fn new() -> Self {
        Self {}
    }
}

// ---- 配置定义 ----

#[derive(Debug, Clone, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum TokenFrom {
    Header,
    Query,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct TokenSource {
    #[serde(default = "default_token_from")]
    pub from: TokenFrom,

    /// Header 名称或 Query 参数名，默认 "Authorization"
    #[serde(default = "default_token_name")]
    pub name: String,

    /// Token 前缀（仅 Header 模式），默认 "Bearer "
    #[serde(default = "default_token_prefix")]
    pub prefix: String,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ValidateConfig {
    /// 期望的签发者（iss）
    pub iss: Option<String>,
    /// 允许的受众列表（aud）
    pub aud: Option<Vec<String>>,
    /// exp/nbf 校验的时钟偏差（秒），默认 0
    #[serde(default)]
    pub leeway: u64,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JwtConfig {
    #[serde(default)]
    pub token_source: TokenSource,

    /// HMAC 签名密钥。提供此值则启用验签，否则只解码不做验签
    pub secret: Option<String>,

    /// 签名算法（HS256/HS384/HS512）。不指定则从 JWT header 中读取
    pub algorithm: Option<String>,

    /// Claims 校验规则
    pub validate: Option<ValidateConfig>,

    /// Claims 注入请求头的映射，如 { "sub": "X-JWT-Subject" }
    #[serde(default)]
    pub claims_to_headers: HashMap<String, String>,

    /// 校验后移除 Authorization header
    #[serde(default)]
    pub strip_token: bool,
}

// ---- 默认值 ----

fn default_token_from() -> TokenFrom {
    TokenFrom::Header
}
fn default_token_name() -> String {
    "Authorization".to_string()
}
fn default_token_prefix() -> String {
    "Bearer ".to_string()
}

impl Default for TokenSource {
    fn default() -> Self {
        Self {
            from: TokenFrom::Header,
            name: "Authorization".to_string(),
            prefix: "Bearer ".to_string(),
        }
    }
}

impl Default for JwtConfig {
    fn default() -> Self {
        Self {
            token_source: TokenSource::default(),
            secret: None,
            algorithm: None,
            validate: None,
            claims_to_headers: HashMap::new(),
            strip_token: false,
        }
    }
}

// ---- Base64url 解码 ----

fn decode_b64url(input: &str) -> Result<Vec<u8>, PluginError> {
    use base64::engine::general_purpose::URL_SAFE_NO_PAD;

    // RFC 4648 §5：base64url 解码缺失填充时自动补齐
    let padded = match input.len() % 4 {
        2 => format!("{}==", input),
        3 => format!("{}=", input),
        _ => input.to_string(),
    };

    URL_SAFE_NO_PAD
        .decode(padded.as_bytes())
        .map_err(|e| PluginError::ExecuteError(format!("Base64 decode failed: {}", e)))
}

// ---- HMAC 验签 ----

fn verify_hmac(
    header_b64: &str,
    payload_b64: &str,
    signature: &[u8],
    secret: &[u8],
    alg: &str,
) -> Result<(), PluginError> {
    use hmac::Mac;

    let data = format!("{}.{}", header_b64, payload_b64);

    match alg.to_uppercase().as_str() {
        "HS256" => {
            let mut mac = hmac::Hmac::<sha2::Sha256>::new_from_slice(secret)
                .map_err(|e| PluginError::ExecuteError(format!("Invalid secret: {}", e)))?;
            mac.update(data.as_bytes());
            mac.verify_slice(signature)
                .map_err(|_| PluginError::ExecuteError("JWT signature mismatch".to_string()))
        }
        "HS384" => {
            let mut mac = hmac::Hmac::<sha2::Sha384>::new_from_slice(secret)
                .map_err(|e| PluginError::ExecuteError(format!("Invalid secret: {}", e)))?;
            mac.update(data.as_bytes());
            mac.verify_slice(signature)
                .map_err(|_| PluginError::ExecuteError("JWT signature mismatch".to_string()))
        }
        "HS512" => {
            let mut mac = hmac::Hmac::<sha2::Sha512>::new_from_slice(secret)
                .map_err(|e| PluginError::ExecuteError(format!("Invalid secret: {}", e)))?;
            mac.update(data.as_bytes());
            mac.verify_slice(signature)
                .map_err(|_| PluginError::ExecuteError("JWT signature mismatch".to_string()))
        }
        _ => Err(PluginError::ExecuteError(format!(
            "Unsupported algorithm: {}. Supported: HS256, HS384, HS512",
            alg
        ))),
    }
}

// ---- Claims 校验 ----

fn current_timestamp() -> u64 {
    SystemTime::now()
        .duration_since(UNIX_EPOCH)
        .unwrap_or_default()
        .as_secs()
}

fn validate_claims(payload: &Value, validate: &ValidateConfig) -> Result<(), PluginError> {
    let now = current_timestamp();

    // exp（过期时间）
    if let Some(exp) = payload.get("exp").and_then(|v| v.as_u64()) {
        if now > exp.saturating_add(validate.leeway) {
            return Err(PluginError::ExecuteError(format!(
                "JWT expired at {}, current time {}",
                exp, now
            )));
        }
    }

    // nbf（生效时间）
    if let Some(nbf) = payload.get("nbf").and_then(|v| v.as_u64()) {
        if now.saturating_add(validate.leeway) < nbf {
            return Err(PluginError::ExecuteError(format!(
                "JWT not yet valid (nbf: {}, current: {})",
                nbf, now
            )));
        }
    }

    // iss（签发者）
    if let Some(ref expected_iss) = validate.iss {
        match payload.get("iss").and_then(|v| v.as_str()) {
            Some(iss) if iss == expected_iss => {}
            Some(iss) => {
                return Err(PluginError::ExecuteError(format!(
                    "JWT iss mismatch: expected '{}', got '{}'",
                    expected_iss, iss
                )));
            }
            None => {
                return Err(PluginError::ExecuteError(format!(
                    "JWT iss claim missing, expected '{}'",
                    expected_iss
                )));
            }
        }
    }

    // aud（受众）
    if let Some(ref expected_aud) = validate.aud {
        let token_aud = payload
            .get("aud")
            .ok_or_else(|| {
                PluginError::ExecuteError(format!(
                    "JWT aud claim missing, expected one of {:?}",
                    expected_aud
                ))
            })?;

        let matched = match token_aud {
            Value::String(s) => expected_aud.contains(s),
            Value::Array(arr) => arr
                .iter()
                .filter_map(|v| v.as_str())
                .any(|a| expected_aud.contains(&a.to_string())),
            _ => false,
        };

        if !matched {
            return Err(PluginError::ExecuteError(format!(
                "JWT aud mismatch: token aud={:?}, expected one of {:?}",
                token_aud, expected_aud
            )));
        }
    }

    Ok(())
}

// ---- 插件实现 ----

#[async_trait]
impl Plugin for JwtValidator {
    fn name(&self) -> &str {
        "jwt-validator"
    }

    fn info(&self) -> PluginInfo {
        PluginInfo {
            version: Version::new(0, 1, 0),
            default_config: json!({
                "token_source": {
                    "from": "header",
                    "name": "Authorization",
                    "prefix": "Bearer "
                },
                "algorithm": "HS256",
                "validate": {
                    "leeway": 60
                },
                "claims_to_headers": {
                    "sub": "X-JWT-Subject"
                },
                "strip_token": false
            }),
            description: "JWT Token 验证与 Claims 提取，支持 HS256/HS384/HS512 验签".to_string(),
            readme: Some(include_str!("../README.md").to_string()),
        }
    }

    async fn on_request(
        &self,
        config: &Value,
        head: &mut request::Parts,
        _ctx: &mut dyn PluginContext,
    ) -> Result<(), PluginError> {
        let cfg: JwtConfig = serde_json::from_value(config.clone()).map_err(|e| {
            PluginError::ExecuteError(format!("Failed to parse JWT config: {}", e))
        })?;

        // 1. 提取 JWT
        let token = match cfg.token_source.from {
            TokenFrom::Header => {
                let header_value = head
                    .headers
                    .get(&cfg.token_source.name)
                    .ok_or_else(|| {
                        PluginError::ExecuteError(format!(
                            "Missing header '{}' for JWT token",
                            cfg.token_source.name
                        ))
                    })?
                    .to_str()
                    .map_err(|e| {
                        PluginError::ExecuteError(format!("Invalid header value: {}", e))
                    })?;

                if !header_value.starts_with(&cfg.token_source.prefix) {
                    return Err(PluginError::ExecuteError(format!(
                        "JWT token prefix mismatch: expected '{}'",
                        cfg.token_source.prefix
                    )));
                }
                header_value[cfg.token_source.prefix.len()..].to_string()
            }
            TokenFrom::Query => {
                let query = head.uri.query().ok_or_else(|| {
                    PluginError::ExecuteError("No query string in request URI".to_string())
                })?;

                query
                    .split('&')
                    .filter_map(|pair| {
                        let mut parts = pair.splitn(2, '=');
                        let key = parts.next()?;
                        let val = parts.next().unwrap_or("");
                        if key == cfg.token_source.name {
                            Some(val.to_string())
                        } else {
                            None
                        }
                    })
                    .next()
                    .ok_or_else(|| {
                        PluginError::ExecuteError(format!(
                            "Missing query parameter '{}' for JWT token",
                            cfg.token_source.name
                        ))
                    })?
            }
        };

        // 2. 拆解 JWT (header.payload.signature)
        let segments: Vec<&str> = token.splitn(3, '.').collect();
        if segments.len() != 3 {
            return Err(PluginError::ExecuteError(
                "Invalid JWT format: expected 3 dot-separated segments".to_string(),
            ));
        }
        let (header_b64, payload_b64, sig_b64) = (segments[0], segments[1], segments[2]);

        // 3. 解码 header
        let header_bytes = decode_b64url(header_b64)?;
        let _header_val: Value = serde_json::from_slice(&header_bytes).map_err(|e| {
            PluginError::ExecuteError(format!("Invalid JWT header JSON: {}", e))
        })?;

        // 4. 解码 payload
        let payload_bytes = decode_b64url(payload_b64)?;
        let payload: Value = serde_json::from_slice(&payload_bytes).map_err(|e| {
            PluginError::ExecuteError(format!("Invalid JWT payload JSON: {}", e))
        })?;

        // 5. 验签（配置了 secret 时启用）
        if let Some(ref secret) = cfg.secret {
            let alg = cfg
                .algorithm
                .as_deref()
                .or_else(|| {
                    _header_val
                        .get("alg")
                        .and_then(|v| v.as_str())
                })
                .ok_or_else(|| {
                    PluginError::ExecuteError(
                        "JWT algorithm not specified in config or JWT header".to_string(),
                    )
                })?;

            let sig_bytes = decode_b64url(sig_b64)?;
            verify_hmac(header_b64, payload_b64, &sig_bytes, secret.as_bytes(), alg)?;
        }

        // 6. Claims 校验
        if let Some(ref validate) = cfg.validate {
            validate_claims(&payload, validate)?;
        }

        // 7. Claims 注入请求头
        for (claim_name, header_name) in &cfg.claims_to_headers {
            if let Some(claim_value) = payload.get(claim_name) {
                let value_str = match claim_value {
                    Value::String(s) => s.clone(),
                    other => other.to_string(),
                };
                head.headers.insert(
                    HeaderName::from_bytes(header_name.as_bytes()).map_err(|e| {
                        PluginError::ExecuteError(format!("Invalid header name: {}", e))
                    })?,
                    HeaderValue::from_str(&value_str).map_err(|e| {
                        PluginError::ExecuteError(format!("Invalid header value: {}", e))
                    })?,
                );
            }
        }

        // 8. 移除原始 Authorization header
        if cfg.strip_token {
            if let TokenFrom::Header = cfg.token_source.from {
                head.headers.remove(&cfg.token_source.name);
            }
        }

        Ok(())
    }


}

export_wasm!(JwtValidator);
