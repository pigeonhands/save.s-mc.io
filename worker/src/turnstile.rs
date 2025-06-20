use serde::{Deserialize, Serialize};

use crate::error::HttpError;

const TURNSTILE_PRIVATE_KEY: Option<&'static str> = option_env!("TURNSTILE_PRIVATE_KEY");

#[derive(Serialize, Deserialize)]
struct RequestParams {
    pub secret: String,
    pub response: String,
}

#[derive(Serialize, Deserialize)]
pub struct ResponseData {
    pub success: bool,
}

pub fn enabled() -> bool {
    TURNSTILE_PRIVATE_KEY.is_some()
}

pub async fn validate(response: String) -> Result<(), HttpError> {
    let key = match TURNSTILE_PRIVATE_KEY {
        Some(key) => key,
        None => {
            log::warn!("Captcha verification disabled");
            return Ok(());
        }
    };

    let pl = RequestParams {
        secret: key.into(),
        response: response.into(),
    };

    let req = worker::send::SendFuture::new(
        reqwest::Client::new()
            .post("https://challenges.cloudflare.com/turnstile/v0/siteverify")
            .json(&pl)
            .send(),
    )
    .await?;

    let resp: ResponseData = worker::send::SendFuture::new(req.json()).await?;

    if resp.success {
        Ok(())
    } else {
        Err(HttpError::BadCaptcha)
    }
}
