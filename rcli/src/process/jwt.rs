use std::ops::Add;
use std::time::{Duration, SystemTime};

use anyhow::Result;
use jsonwebtoken::{decode, encode, Algorithm, DecodingKey, EncodingKey, Header, Validation};
use serde::{Deserialize, Serialize};

use crate::{get_reader, JwtKeyType};

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Claim {
    sub: String,
    aud: String,
    exp: u64,
}

pub fn process_jwt_encode(
    key: &str,
    key_type: JwtKeyType,
    sub: &str,
    aud: &str,
    exp: Duration,
    algorithm: Algorithm,
) -> Result<String> {
    let mut reader = get_reader(key)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let key = match key_type {
        JwtKeyType::Secret => EncodingKey::from_secret(buf.as_ref()),
        JwtKeyType::Base64Secret => EncodingKey::from_base64_secret(std::str::from_utf8(&buf)?)?,
        JwtKeyType::Rsa => EncodingKey::from_rsa_pem(buf.as_ref())?,
        JwtKeyType::ECDSA => EncodingKey::from_ec_pem(buf.as_ref())?,
        JwtKeyType::EdDSA => EncodingKey::from_ed_pem(buf.as_ref())?,
        JwtKeyType::RsaDer => EncodingKey::from_rsa_der(buf.as_ref()),
        JwtKeyType::ECDSADer => EncodingKey::from_ec_der(buf.as_ref()),
        JwtKeyType::EdDSADer => EncodingKey::from_ed_der(buf.as_ref()),
    };

    let expiration = SystemTime::now()
        .duration_since(SystemTime::UNIX_EPOCH)
        .unwrap()
        .add(exp)
        .as_secs();

    let claim = Claim {
        sub: sub.to_string(),
        aud: aud.to_string(),
        exp: expiration,
    };

    let token = encode(&Header::new(algorithm), &claim, &key)?;
    Ok(token)
}

pub fn process_jwt_decode(
    key: &str,
    key_type: JwtKeyType,
    token: &str,
    algorithm: Algorithm,
    aud: &str,
) -> Result<Claim> {
    let mut reader = get_reader(key)?;
    let mut buf = Vec::new();
    reader.read_to_end(&mut buf)?;
    let key = match key_type {
        JwtKeyType::Secret => DecodingKey::from_secret(buf.as_ref()),
        JwtKeyType::Base64Secret => DecodingKey::from_base64_secret(std::str::from_utf8(&buf)?)?,
        JwtKeyType::Rsa => DecodingKey::from_rsa_pem(buf.as_ref())?,
        JwtKeyType::ECDSA => DecodingKey::from_ec_pem(buf.as_ref())?,
        JwtKeyType::EdDSA => DecodingKey::from_ed_pem(buf.as_ref())?,
        JwtKeyType::RsaDer => DecodingKey::from_rsa_der(buf.as_ref()),
        JwtKeyType::ECDSADer => DecodingKey::from_ec_der(buf.as_ref()),
        JwtKeyType::EdDSADer => DecodingKey::from_ed_der(buf.as_ref()),
    };

    let mut validation = Validation::new(algorithm);
    if aud.is_empty() {
        validation.validate_aud = false;
    } else {
        validation.set_audience(&[aud]);
    }

    let token = decode::<Claim>(token, &key, &validation)?;
    Ok(token.claims)
}

#[cfg(test)]
mod tests {
    use humantime::parse_duration;

    use super::*;

    #[test]
    fn test_sign_verify() {
        let key = "fixtures/jwt-secret.txt";
        let key_type = JwtKeyType::Secret;
        let sub = "user";
        let aud = "admin";
        let exp = parse_duration("14d").unwrap();

        let token = process_jwt_encode(key, key_type, sub, aud, exp, Algorithm::HS256).unwrap();
        let claim = process_jwt_decode(key, key_type, token.as_str(), Algorithm::HS256, aud);
        assert!(claim.is_ok());
        let claim = claim.unwrap();
        assert_eq!(claim.sub, sub);
        assert_eq!(claim.aud, aud);
    }
}
