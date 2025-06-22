use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};

use crate::{
    error::{HttpError, HttpResult},
    turnstile,
};

use super::context::AppState;

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaResponse;

impl FromRequestParts<AppState> for CaptchaResponse {
    type Rejection = crate::error::HttpError;

    async fn from_request_parts(parts: &mut Parts, state: &AppState) -> HttpResult<Self> {
        let turnstile_key = match state.turnstile_private_key() {
            Some(key) => key,
            None => {
                log::warn!("turnstile is disabled");
                return Ok(Self);
            }
        };

        let auth_header = parts
            .headers
            .get("Captcha-Response")
            .ok_or(HttpError::BadCaptcha)?;

        let captcha_response = auth_header.to_str().map_err(|_| {
            log::debug!("Captcha-Response header is not UTF-8");
            HttpError::BadCaptcha
        })?;

        turnstile::validate(captcha_response.into(), turnstile_key).await?;

        Ok(Self)
    }
}
