use axum::extract::FromRequestParts;
use axum::http::request::Parts;
use serde::{Deserialize, Serialize};

use crate::{
    error::{HttpError, HttpResult},
    turnstile,
};

#[derive(Debug, Serialize, Deserialize)]
pub struct CaptchaResponse;

impl FromRequestParts<()> for CaptchaResponse {
    type Rejection = crate::error::HttpError;

    async fn from_request_parts(parts: &mut Parts, _: &()) -> HttpResult<Self> {
        if !turnstile::enabled() {
            return Ok(Self);
        }

        let auth_header = parts
            .headers
            .get("Captcha-Response")
            .ok_or(HttpError::BadCaptcha)?;

        let captcha_response = auth_header.to_str().map_err(|_| {
            log::debug!("Captcha-Response header is not UTF-8");
            HttpError::BadCaptcha
        })?;

        turnstile::validate(captcha_response.into()).await?;

        Ok(Self)
    }
}
