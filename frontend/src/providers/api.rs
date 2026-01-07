use anyhow::bail;
use common::{PublicKeyRequest, PublicKeyResponse, RegisterBeginRequest, RegisterBeginResponse};
use gloo::net::http::{Request, RequestBuilder};
use struct_iterable::Iterable;

use crate::components::turnstile;

trait ToRequestParams {
    fn to_request_params(&self) -> impl IntoIterator<Item = (&'static str, String)>;
}

impl<T> ToRequestParams for T
where
    T: Iterable,
{
    fn to_request_params(&self) -> impl IntoIterator<Item = (&'static str, String)> {
        self.iter()
            .map(|(name, value)| (name, value.downcast_ref::<String>().unwrap().clone()))
    }
}

fn get(url: &str) -> RequestBuilder {
    let builder = Request::get(url);

    let builder = if let Some(token) = turnstile::response().as_deref() {
        builder.header("Captcha-Response", token)
    } else {
        builder
    };
    builder
}

fn post(url: &str) -> RequestBuilder {
    let builder = Request::post(url);

    let builder = if let Some(token) = turnstile::response().as_deref() {
        builder.header("Captcha-Response", token)
    } else {
        builder
    };
    builder
}

pub async fn get_public_key(email: String) -> anyhow::Result<PublicKeyResponse> {
    let test = PublicKeyRequest {
        email: email.clone(),
    };

    let resp = get("/api/save/public-key")
        .query(
            PublicKeyRequest {
                email: email.clone(),
            }
            .to_request_params(),
        )
        .send()
        .await?;

    if resp.status() == 404 {
        bail!("Public key for email '{}' not found", email);
    }

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

pub async fn begin_registration(
    email: String,
    encryption_key: String,
    pub_key: String,
) -> anyhow::Result<RegisterBeginResponse> {
    let resp = post("/api/register/begin")
        .json(&RegisterBeginRequest {
            email,
            encryption_key,
            pub_key,
        })?
        .send()
        .await?;

    if resp.status() == 404 {
        bail!("Public key for email not found");
    }

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
