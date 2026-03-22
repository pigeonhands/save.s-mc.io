use anyhow::bail;
use common::{
    AssertionResponse, AuthBeginRequest, AuthBeginResponse, AuthFinishRequest, AuthFinishResponse,
    PublicKeyRequest, PublicKeyResponse, ReadItemsResponse, RegisterBeginRequest,
    RegisterBeginResponse, RegisterFinishRequest, RegisterFinishResponse, RegisterPublicKeyCredential,
};
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
    let _test = PublicKeyRequest {
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

    Ok(resp.json().await?)
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
            "Failed to begin registration ({}): {}",
            resp.status(),
            resp.status_text()
        );
    }

    Ok(resp.json().await?)
}

pub async fn finish_registration(
    email: String,
    credential: RegisterPublicKeyCredential,
) -> anyhow::Result<RegisterFinishResponse> {
    let resp = post("/api/register/finish")
        .json(&RegisterFinishRequest { email, credential })?
        .send()
        .await?;

    if resp.status() == 401 {
        anyhow::bail!("Registration session expired. Please start over.");
    }

    if resp.status() != 200 {
        anyhow::bail!(
            "Registration failed ({}): {}",
            resp.status(),
            resp.status_text()
        );
    }

    Ok(resp.json().await?)
}

pub async fn auth_begin(email: String) -> anyhow::Result<AuthBeginResponse> {
    let resp = post("/api/auth/begin")
        .json(&AuthBeginRequest { email })?
        .send()
        .await?;

    if resp.status() == 401 {
        anyhow::bail!("No registered security key found for this email.");
    }

    if resp.status() != 200 {
        anyhow::bail!("Auth begin failed ({}): {}", resp.status(), resp.status_text());
    }

    Ok(resp.json().await?)
}

pub async fn auth_finish(
    email: String,
    assertion: AssertionResponse,
) -> anyhow::Result<AuthFinishResponse> {
    let resp = post("/api/auth/finish")
        .json(&AuthFinishRequest { email, assertion })?
        .send()
        .await?;

    if resp.status() == 401 {
        anyhow::bail!("Authentication failed. Wrong security key or expired session.");
    }

    if resp.status() != 200 {
        anyhow::bail!("Auth finish failed ({}): {}", resp.status(), resp.status_text());
    }

    Ok(resp.json().await?)
}

pub async fn read_items(session_token: &str) -> anyhow::Result<ReadItemsResponse> {
    let resp = Request::get("/api/read/items")
        .header("Authorization", &format!("Bearer {session_token}"))
        .send()
        .await?;

    if resp.status() == 401 {
        anyhow::bail!("Session expired. Please log in again.");
    }

    if resp.status() != 200 {
        anyhow::bail!("Failed to load items ({}): {}", resp.status(), resp.status_text());
    }

    Ok(resp.json().await?)
}
