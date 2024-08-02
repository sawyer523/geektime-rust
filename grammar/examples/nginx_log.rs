use anyhow::{anyhow, Result};
use regex::Regex;

#[derive(Debug)]
struct NginxLog {
    addr: String,
    datetime: String,
    method: String,
    url: String,
    protocol: String,
    status: usize,
    referer: String,
    user_agent: String,
}

fn main() -> Result<()> {
    let str = &r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
    let log = parse_nginx_log(str)?;
    println!("{:?}", log);
    Ok(())
}

fn parse_nginx_log(s: &str) -> Result<NginxLog> {
    let re = Regex::new(
        r#"^(?<ip>\S+)\s+\S+\s+\S+\s+\[(?<date>[^\]]+)\]\s+"(?<method>\S+)\s+(?<url>\S+)\s+(?<proto>[^"]+)"\s+(?<status>\d+)\s+(?<referer>\d+)\s+\S+\s+"(?<us>[^"]+)"$"#,
    )?;
    let cap = re.captures(s).ok_or(anyhow!("parse error"))?;

    let addr = cap
        .name("ip")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse ip error"))?;
    let datetime = cap
        .name("date")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse date error"))?;
    let method = cap
        .name("method")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse method error"))?;
    let url = cap
        .name("url")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse url error"))?;
    let protocol = cap
        .name("proto")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse protocol error"))?;
    let status = cap
        .name("status")
        .map(|m| m.as_str().parse::<usize>())
        .unwrap_or(Ok(0))?;
    let referer = cap
        .name("referer")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse referer error"))?;
    let user_agent = cap
        .name("us")
        .map(|m| m.as_str().to_string())
        .ok_or(anyhow!("parse user_agent error"))?;

    Ok(NginxLog {
        addr,
        datetime,
        method,
        url,
        protocol,
        status,
        referer,
        user_agent,
    })
}
