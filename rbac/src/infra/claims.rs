use anyhow::{Context, Ok};
use chrono::{Duration, Utc};
use jsonwebtoken::{DecodingKey, EncodingKey, Header, Validation, decode, encode};
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize, Deserialize)]
pub struct Claims {
    pub sub: String,
    pub iat: usize,
    pub exp: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct Token {
    pub token_str: String,
    pub iat: usize,
    pub exp: usize,
}

pub fn generate_jwt(user_id: &str, secret: &str) -> anyhow::Result<Token> {
    let now = Utc::now();
    let iat = now.timestamp() as usize;
    let exp = (now + Duration::hours(2)).timestamp() as usize;
    let claims = Claims {
        sub: user_id.to_owned(),
        iat,
        exp,
    };

    let token_str = encode(
        &Header::new(jsonwebtoken::Algorithm::HS256),
        &claims,
        &EncodingKey::from_secret(secret.as_bytes()),
    )
    .context("jwt 生成失败")?;

    Ok(Token {
        token_str: token_str,
        iat,
        exp,
    })
}

pub fn validate_jwt(token: &str, secret: &str) -> anyhow::Result<Claims> {
    let mut validation = Validation::new(jsonwebtoken::Algorithm::HS256);

    validation.set_required_spec_claims(&["exp", "sub"]);

    let token_data = decode::<Claims>(
        token,
        &DecodingKey::from_secret(secret.as_bytes()),
        &validation,
    )?;

    Ok(token_data.claims)
}
