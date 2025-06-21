use std::sync::Arc;

use axum::extract::FromRef;
use worker::{D1Database, Env, send::SendWrapper};

type WrappedState<T> = Arc<SendWrapper<T>>;

pub struct InnerContext {
    env: Env,
}

#[derive(Clone)]
pub struct AppState(WrappedState<Env>);

impl AppState {
    pub fn from_env(env: Env) -> Self {
        Self(Arc::new(SendWrapper::new(env)))
    }
}

pub struct DbCtx(SendWrapper<D1Database>);

impl FromRef<AppState> for DbCtx {
    fn from_ref(input: &AppState) -> Self {
        let db = input.0.d1("DB").expect("Cant get db binding");

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
