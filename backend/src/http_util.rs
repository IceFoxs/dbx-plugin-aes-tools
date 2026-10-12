use std::sync::OnceLock;
use std::time::Duration;

/// 默认超时时间（秒）
const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// 全局复用的 Agent（内部带连接池）
static AGENT: OnceLock<ureq::Agent> = OnceLock::new();

fn agent() -> &'static ureq::Agent {
    AGENT.get_or_init(|| {
        ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(DEFAULT_TIMEOUT_SECS))
            .build()
    })
}

pub struct HttpUtil;

impl HttpUtil {
    /// GET 请求（默认超时），返回 String
    pub fn get(url: &str) -> Result<String, Box<dyn std::error::Error>> {
        let body = agent().get(url).call()?.into_string()?;
        Ok(body)
    }

    /// GET 请求（自定义超时，单位秒），返回 String
    pub fn get_with_timeout(
        url: &str,
        timeout_secs: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // ureq 的超时是 Agent 级别的，想每请求不同就临时建一个 Agent
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build();
        let body = agent.get(url).call()?.into_string()?;
        Ok(body)
    }

    /// GET 请求（带请求头 + 自定义超时）
    pub fn get_with_headers(
        url: &str,
        headers: Vec<(&str, &str)>,
        timeout_secs: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build();
        let mut req = agent.get(url);
        for (k, v) in headers {
            req = req.set(k, v);
        }
        Ok(req.call()?.into_string()?)
    }

    /// GET 请求（带查询参数 + 自定义超时）
    pub fn get_with_params(
        url: &str,
        params: Vec<(&str, &str)>,
        timeout_secs: u64,
    ) -> Result<String, Box<dyn std::error::Error>> {
        let agent = ureq::AgentBuilder::new()
            .timeout(Duration::from_secs(timeout_secs))
            .build();
        let mut req = agent.get(url);
        for (k, v) in params {
            req = req.query(k, v);
        }
        Ok(req.call()?.into_string()?)
    }
}