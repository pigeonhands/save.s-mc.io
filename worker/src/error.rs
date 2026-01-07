use axum::http::header::WWW_AUTHENTICATE;
use axum::http::{HeaderMap, HeaderValue, StatusCode};
use axum::response::{IntoResponse, Response};

pub type HttpResult<T> = std::result::Result<T, HttpError>;

#[derive(thiserror::Error, Debug)]
pub enum HttpError {
    #[error("authentication required")]
    Unauthorized,

    #[error("user may not perform that action")]
    Forbidden,

    #[error("user failed the captcha")]
    BadCaptcha,

    #[error("request path not found")]
    NotFound,

    #[error("error in the request body")]
    UnprocessableEntity,

    #[error("Internal http error. {0:?}")]
    HttpError(reqwest::Error),

    #[error("Internal worker error. {0:?}")]
    WorkerError(worker::Error),

    #[error("an internal server error occurred. {0:?}")]
    Anyhow(#[from] anyhow::Error),
}

impl HttpError {
    fn status_code(&self) -> StatusCode {
        match self {
            Self::Unauthorized => StatusCode::UNAUTHORIZED,
            Self::Forbidden => StatusCode::FORBIDDEN,
            Self::BadCaptcha => StatusCode::BAD_REQUEST,
            Self::NotFound => StatusCode::NOT_FOUND,
            Self::HttpError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::WorkerError(_) => StatusCode::INTERNAL_SERVER_ERROR,
            Self::UnprocessableEntity { .. } => StatusCode::UNPROCESSABLE_ENTITY,
            Self::Anyhow(_) => StatusCode::INTERNAL_SERVER_ERROR,
        }
    }
}

impl IntoResponse for HttpError {
    fn into_response(self) -> Response {
        match self {
            Self::Unauthorized => {
                return (
                    self.status_code(),
                    [(WWW_AUTHENTICATE, HeaderValue::from_static("Token"))]
                        .into_iter()
                        .collect::<HeaderMap>(),
                    self.to_string(),
                )
                    .into_response();
            }

            Self::Anyhow(ref e) => {
                log::error!("Generic error: {:?}", e);
            }

            // Other errors get mapped normally.
            _ => (),
        }

        (self.status_code(), self.to_string()).into_response()
    }
}

impl From<reqwest::Error> for HttpError {
    fn from(value: reqwest::Error) -> Self {
        Self::HttpError(value)
    }
}

impl From<worker::Error> for HttpError {
    fn from(value: worker::Error) -> Self {
        Self::WorkerError(value)
    }
}
