use anyhow::bail;
use common::{PublicKeyRequest, PublicKeyResponse};
use gloo::net::http::{Request, RequestBuilder};

fn get(url: &str, captcha_result: Option<&str>) -> RequestBuilder {
    let builder = Request::get(url);

    let builder = if let Some(token) = captcha_result {
        builder.header("Captcha-Response", token)
    } else {
        builder
    };
    builder
}

pub async fn get_public_key(
    email: String,
    captcha_result: Option<String>,
) -> anyhow::Result<PublicKeyResponse> {
    let resp = get("/api/save/public-key", captcha_result.as_deref())
        .query(
            PublicKeyRequest {
                email: email.into(),
            }
            .iter(),
        )
        .send()
        .await?;

    if resp.status() != 200 {
        bail!(
            "Failed to get pub key ({}). Api returned {}",
            resp.status_text(),
            resp.status()
        );
    }

    let json_resp = resp.json().await?;

    Ok(json_resp)
}
