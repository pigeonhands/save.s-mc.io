use axum::{extract::FromRef, http::Uri};
use common::PublicKeyCredentialCreationOptions;

use crate::http::context::AppState;
pub struct PasskeyCtx {
    uri: Uri,
}

impl FromRef<AppState> for PasskeyCtx {
    fn from_ref(input: &AppState) -> Self {
        Self {
            uri: input.uri.clone(),
        }
    }
}

impl PasskeyCtx {
    pub fn start_register(
        &self,
        email: &str,
    ) -> anyhow::Result<PublicKeyCredentialCreationOptions> {
        todo!()
    }
}
