use std::fmt;
use std::str::FromStr;
use std::time::Duration;

use clap::Parser;
use enum_dispatch::enum_dispatch;
use jsonwebtoken::errors::Error;
use jsonwebtoken::Algorithm;

use crate::{process_jwt_decode, process_jwt_encode, CmdExecutor};

use super::verify_file;

#[derive(Debug, Parser)]
#[enum_dispatch(CmdExecutor)]
pub enum JwtSubCommand {
    #[command(about = "Generate a JWT from a claim.")]
    Sign(JwtSignOpts),

    #[command(about = "Verify a JWT")]
    Verify(JwtVerifyOpts),
}

#[derive(Debug, Parser)]
pub struct JwtSignOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub key: String,

    #[arg(long, value_parser = parse_key_type, default_value = "secret")]
    pub key_type: JwtKeyType,

    #[arg(long)]
    pub sub: String,

    #[arg(long)]
    pub aud: String,

    #[arg(long, value_parser = humantime::parse_duration)]
    pub exp: Duration,

    #[arg(long, value_parser = parse_algorithm, default_value = "HS256")]
    pub algorithm: Algorithm,
}

#[derive(Debug, Parser)]
pub struct JwtVerifyOpts {
    #[arg(short, long, value_parser = verify_file, default_value = "-")]
    pub key: String,

    #[arg(long, value_parser = parse_key_type, default_value = "secret")]
    pub key_type: JwtKeyType,

    #[arg(long)]
    pub token: String,

    #[arg(long, value_parser = parse_algorithm, default_value = "HS256")]
    pub algorithm: Algorithm,

    #[arg(long, default_value = "")]
    pub aud: String,
}

#[derive(Debug, Clone, Copy)]
pub enum JwtKeyType {
    Secret,
    Base64Secret,
    Rsa,
    ECDSA,
    EdDSA,
    RsaDer,
    ECDSADer,
    EdDSADer,
}

fn parse_key_type(s: &str) -> Result<JwtKeyType, anyhow::Error> {
    s.parse()
}

fn parse_algorithm(s: &str) -> Result<Algorithm, Error> {
    s.parse()
}

impl FromStr for JwtKeyType {
    type Err = anyhow::Error;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        match s.to_lowercase().as_str() {
            "secret" => Ok(Self::Secret),
            "base64-secret" => Ok(Self::Base64Secret),
            "rsa" => Ok(Self::Rsa),
            "ecdsa" => Ok(Self::ECDSA),
            "eddsa" => Ok(Self::EdDSA),
            "rsa-der" => Ok(Self::RsaDer),
            "ecdsa-der" => Ok(Self::ECDSADer),
            "eddsa-der" => Ok(Self::EdDSADer),
            _ => Err(anyhow::anyhow!("invalid key type")),
        }
    }
}

impl From<JwtKeyType> for &str {
    fn from(key_type: JwtKeyType) -> Self {
        match key_type {
            JwtKeyType::Secret => "secret",
            JwtKeyType::Base64Secret => "base64Secret",
            JwtKeyType::Rsa => "rsa",
            JwtKeyType::ECDSA => "ecdsa",
            JwtKeyType::EdDSA => "eddsa",
            JwtKeyType::RsaDer => "rsaDer",
            JwtKeyType::ECDSADer => "ecdsaDer",
            JwtKeyType::EdDSADer => "eddsaDer",
        }
    }
}

impl fmt::Display for JwtKeyType {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", Into::<&'static str>::into(*self))
    }
}

impl CmdExecutor for JwtSignOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let signed = process_jwt_encode(
            &self.key,
            self.key_type,
            &self.sub,
            &self.aud,
            self.exp,
            self.algorithm,
        )?;
        println!("{}", signed);
        Ok(())
    }
}

impl CmdExecutor for JwtVerifyOpts {
    async fn execute(self) -> anyhow::Result<()> {
        let claim = process_jwt_decode(
            &self.key,
            self.key_type,
            &self.token,
            self.algorithm,
            &self.aud,
        );
        println!("Verified: {}", claim.is_ok());
        println!("{:?}", claim?);
        Ok(())
    }
}
