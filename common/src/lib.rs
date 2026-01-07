pub use passkey_types::webauthn::PublicKeyCredentialCreationOptions;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;

#[derive(Clone, Debug, Serialize, Deserialize, Iterable)]
pub struct PublicKeyRequest {
    pub email: String,
}

#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct PublicKeyResponse {
    pub email: String,
    pub pub_key: String,
}

#[derive(Clone, Debug, Serialize, Deserialize, Iterable)]
pub struct RegisterBeginRequest {
    pub email: String,
    pub encryption_key: String,
    pub pub_key: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterBeginResponse {
    pub passkey_challenge: PublicKeyCredentialCreationOptions,
    pub pgp_channenge: String,
}
