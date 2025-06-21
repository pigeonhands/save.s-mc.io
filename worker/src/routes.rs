use common::{PublicKeyRequest, PublicKeyResponse};

use crate::error::HttpResult;
use axum::{Json, Router, extract::Query, response::IntoResponse, routing::get};

pub fn router() -> Router {
    Router::new().route("/public-key", get(get_public_key))
}

static DEBUG_KEY: &'static str = r#"
-----BEGIN PGP PUBLIC KEY BLOCK-----

mDMEZ8zPYhYJKwYBBAHaRw8BAQdAPZIcUd/BGPEuiUp3e2U7dIWFl+n4DsOFjXj2
dGhGA4q0E1NhbSBNIDxzYW1Acy1tYy5pbz6IlAQTFgoAPBYhBDTql6996KUygFkd
o1Qh3Pq6DpKiBQJnzM9iAhsDBQkFo5qABAsJCAcEFQoJCAUWAgMBAAIeBQIXgAAK
CRBUIdz6ug6SoqPnAQCyeTu+5J7n/nDDI1G1glQisdY07A1hotef0LwbYUvKqwD/
XrNvzYLPRf359FHbTwxElnf8PEknEUGf3M86+oxvYgm4OARnzM9iEgorBgEEAZdV
AQUBAQdAlTvNg+yH+A4JJg9UF/IPSCc+YAUTetg2BV680xTUvQoDAQgHiH4EGBYK
ACYWIQQ06pevfeilMoBZHaNUIdz6ug6SogUCZ8zPYgIbDAUJBaOagAAKCRBUIdz6
ug6Soh3yAQDFbDN9l9qO5cP/XQyyJb+X8qHw0sr9H83fmeQuwGk3UgD9Gi2p61IS
MnoqIPI3PoRPupuvEpS1U9pFt3rgkQelogS4MwRoUU9SFgkrBgEEAdpHDwEBB0BH
0Arixqe7C9xq0kjJL/NlsVkgM8pqeWsWPzUZIPvZ+Ij1BBgWCgAmFiEENOqXr33o
pTKAWR2jVCHc+roOkqIFAmhRT1ICGwIFCQWjmoAAgQkQVCHc+roOkqJ2IAQZFgoA
HRYhBMXcl3UaCYB98ur3c8+BxxUzcS3LBQJoUU9SAAoJEM+BxxUzcS3LKp4BAIqt
WADQS6K25PsczlhRHS2RrxN0OqQDDDlnlVBW2tM1AP9PapZA8EcPm7113cNiGbWx
fr/8cV2vm52TZ0ATI+0KA6GRAP9/6TvjM0FnyaZppEZV7epXwTW2PkquUpDu7LKw
W+rfJQD/biS/tV9TRxiG91IWwi5ETzexjPCdG+AINYFl80MHugW4MwRoUdgpFgkr
BgEEAdpHDwEBB0CGbOhHgcyJif7P5wXiebzlu8sYkmUl/K+xbw8bNh07xIh+BBgW
CgAmFiEENOqXr33opTKAWR2jVCHc+roOkqIFAmhR2CkCGyAFCQWjmoAACgkQVCHc
+roOkqLOgQD9FPXGjjrhZjoKWQmWmJOGKw24v0bh2cKq94Fqt3hhwwgBAKuThIFg
qc4Y/iqG99yF2DwV/RmyaRRqqjeALeBgF3IB
=rRWJ
-----END PGP PUBLIC KEY BLOCK-----
"#;

#[axum::debug_handler]
pub async fn get_public_key(
    _captcha: CaptchaResponse,
    params: Query<PublicKeyRequest>,
) -> HttpResult<impl IntoResponse> {
    Ok(Json(PublicKeyResponse {
        email: params.email.clone(),
        pub_key: DEBUG_KEY.into(),
    }))
}
