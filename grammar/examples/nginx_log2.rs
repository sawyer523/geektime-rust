use std::net::{IpAddr, Ipv4Addr};
use std::str::FromStr;

use anyhow::{anyhow, Result};
use chrono::{DateTime, Utc};
use winnow::{Parser, PResult};
// Import the FromStr trait
use winnow::ascii::{digit1, space0};
use winnow::combinator::{alt, delimited, separated};
use winnow::token::take_until;

#[allow(unused)]
#[derive(Debug)]
struct NginxLog {
    addr: IpAddr,
    datetime: DateTime<Utc>,
    method: HttpMethod,
    url: String,
    protocol: HttpProtocol,
    status: u16,
    body_bytes: u64,
    referer: String,
    user_agent: String,
}

#[derive(Debug, PartialEq)]
enum HttpMethod {
    Get,
    Post,
    Put,
    Delete,
    Head,
    Options,
    Connect,
    Trace,
    Patch,
}

#[derive(Debug, PartialEq)]
enum HttpProtocol {
    HTTP1_0,
    HTTP1_1,
    HTTP2_0,
    HTTP3_0,
}

fn main() -> Result<()> {
    let str = &r#"93.180.71.3 - - [17/May/2015:08:05:32 +0000] "GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
    let log = parse_nginx_log(str).map_err(|e| anyhow!("Failed to parse log: {:?}", e))?;
    println!("{:?}", log);
    Ok(())
}

fn parse_nginx_log(s: &str) -> PResult<NginxLog> {
    let input = &mut s.as_ref();
    let mut parser = (
        parse_ip,
        parse_ignored,
        parse_ignored,
        parse_datetime,
        parse_http,
        parse_status,
        parse_body_bytes,
        parse_quote_string,
        parse_quote_string,
    );

    let ret = parser.parse_next(input)?;
    Ok(NginxLog {
        addr: ret.0,
        datetime: ret.3,
        method: ret.4 .0,
        url: ret.4 .1,
        protocol: ret.4 .2,
        status: ret.5,
        body_bytes: ret.6,
        referer: ret.7,
        user_agent: ret.8,
    })
}

fn parse_ip(s: &mut &str) -> PResult<IpAddr> {
    let ret: Vec<u8> = separated(4, digit1.parse_to::<u8>(), ".").parse_next(s)?;
    space0(s)?;
    Ok(IpAddr::V4(Ipv4Addr::new(ret[0], ret[1], ret[2], ret[3])))
}

fn parse_datetime(s: &mut &str) -> PResult<DateTime<Utc>> {
    let ret = delimited('[', take_until(1.., ']'), ']').parse_next(s)?;
    space0(s)?;
    let date = DateTime::parse_from_str(&ret, "%d/%b/%Y:%H:%M:%S %z").unwrap();
    Ok(date.with_timezone(&Utc))
}

fn parse_http(s: &mut &str) -> PResult<(HttpMethod, String, HttpProtocol)> {
    let parser = (parse_method, parse_url, parse_protocol);
    let ret = delimited('"', parser, '"').parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_method(s: &mut &str) -> PResult<HttpMethod> {
    let ret = alt((
        "GET", "POST", "PUT", "DELETE", "HEAD", "OPTIONS", "CONNECT", "TRACE", "PATCH",
    ))
    .parse_to()
    .parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_url(s: &mut &str) -> PResult<String> {
    let ret = take_until(1.., ' ').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

fn parse_protocol(s: &mut &str) -> PResult<HttpProtocol> {
    let ret = alt(("HTTP/1.0", "HTTP/1.1", "HTTP/2.0", "HTTP/3.0"))
        .parse_to()
        .parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_status(s: &mut &str) -> PResult<u16> {
    let ret = digit1.parse_to().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_body_bytes(s: &mut &str) -> PResult<u64> {
    let ret = digit1.parse_to().parse_next(s)?;
    space0(s)?;
    Ok(ret)
}

fn parse_quote_string(s: &mut &str) -> PResult<String> {
    let ret = delimited('"', take_until(1.., '"'), '"').parse_next(s)?;
    space0(s)?;
    Ok(ret.to_string())
}

fn parse_ignored(s: &mut &str) -> PResult<()> {
    "- ".parse_next(s)?;
    Ok(())
}

impl FromStr for HttpMethod {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "GET" => Ok(HttpMethod::Get),
            "POST" => Ok(HttpMethod::Post),
            "PUT" => Ok(HttpMethod::Put),
            "DELETE" => Ok(HttpMethod::Delete),
            "HEAD" => Ok(HttpMethod::Head),
            "OPTIONS" => Ok(HttpMethod::Options),
            "CONNECT" => Ok(HttpMethod::Connect),
            "TRACE" => Ok(HttpMethod::Trace),
            "PATCH" => Ok(HttpMethod::Patch),
            _ => Err(anyhow!("parse method error")),
        }
    }
}

impl FromStr for HttpProtocol {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self> {
        match s {
            "HTTP/1.0" => Ok(HttpProtocol::HTTP1_0),
            "HTTP/1.1" => Ok(HttpProtocol::HTTP1_1),
            "HTTP/2.0" => Ok(HttpProtocol::HTTP2_0),
            "HTTP/3.0" => Ok(HttpProtocol::HTTP3_0),
            _ => Err(anyhow!("parse protocol error")),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_parse_ip() {
        let mut s = "93.180.71.3 - -";
        let ret = parse_ip(&mut s);
        assert_eq!(ret, Ok(IpAddr::V4(Ipv4Addr::new(93, 180, 71, 3))));
    }

    #[test]
    fn test_parse_datetime() {
        let mut s = "[17/May/2015:08:05:32 +0000] \"GET /downloads/product_1 HTTP/1.1\" 304 0 \"-\" \"Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)\"";
        let ret = parse_datetime(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap().to_rfc3339(), "2015-05-17T08:05:32+00:00");
    }

    #[test]
    fn test_parse_http() {
        let mut s = r#""GET /downloads/product_1 HTTP/1.1" 304 0 "-" "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)""#;
        let ret = parse_http(&mut s);
        println!("{:?}", ret);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(
            ret.unwrap(),
            (
                HttpMethod::Get,
                "/downloads/product_1".to_string(),
                HttpProtocol::HTTP1_1
            )
        );
    }

    #[test]
    fn test_parse_method() {
        let mut s = "GET /downloads/product_1 HTTP/1.1";
        let ret = parse_method(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap(), HttpMethod::Get);
    }

    #[test]
    fn test_parse_url() {
        let mut s = "/downloads/product_1 HTTP/1.1";
        let ret = parse_url(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap(), "/downloads/product_1".to_string());
    }

    #[test]
    fn test_parse_protocol() {
        let mut s = "HTTP/1.1";
        let ret = parse_protocol(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap(), HttpProtocol::HTTP1_1);
    }

    #[test]
    fn test_parse_status() {
        let mut s = "304 0 \"-\" \"Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)\"";
        let ret = parse_status(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap(), 304);
    }

    #[test]
    fn test_parse_body_bytes() {
        let mut s = "0 \"-\" \"Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)\"";
        let ret = parse_body_bytes(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap(), 0);
    }

    #[test]
    fn test_parse_referer() {
        let mut s = "\"-\" \"Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)\"";
        let ret = parse_quote_string(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(ret.unwrap(), "-".to_string());
    }

    #[test]
    fn test_parse_user_agent() {
        let mut s = "\"Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)\"";
        let ret = parse_quote_string(&mut s);
        assert_eq!(ret.is_ok(), true);
        assert_eq!(
            ret.unwrap(),
            "Debian APT-HTTP/1.3 (0.8.16~exp12ubuntu10.21)".to_string()
        );
    }
}
