use std::sync::Arc;

use axum::{extract::FromRef, http::Uri};
use worker::{D1Database, Env, send::SendWrapper};

type WrappedState<T> = Arc<SendWrapper<T>>;

pub struct InnerContext {
    env: Env,
}

#[derive(Clone)]
pub struct AppState {
    pub env: WrappedState<Env>,
    pub uri: Uri,
}

impl AppState {
    pub fn from_env(env: Env, request: &worker::HttpRequest) -> Self {
        Self {
            env: Arc::new(SendWrapper::new(env)),
            uri: request.uri().clone(),
        }
    }

    pub fn turnstile_private_key(&self) -> Option<String> {
        self.env
            .var("TURNSTILE_PRIVATE_KEY")
            .map(|var| var.to_string())
            .ok()
    }
}

pub struct DbCtx(SendWrapper<D1Database>);

impl FromRef<AppState> for DbCtx {
    fn from_ref(input: &AppState) -> Self {
        let db = input.env.d1("DB").expect("Cant get db binding");

        Self(SendWrapper::new(db))
    }
}
impl AsRef<D1Database> for DbCtx {
    fn as_ref(&self) -> &D1Database {
        &self.0
    }
}

impl std::ops::Deref for DbCtx {
    type Target = D1Database;

    fn deref(&self) -> &Self::Target {
        self.as_ref()
    }
}
