pub use passkey_types::webauthn::PublicKeyCredentialCreationOptions;
pub use passkey_types::webauthn::PublicKeyCredentialRequestOptions;
use serde::{Deserialize, Serialize};
use struct_iterable::Iterable;
pub use webauthn_rs_proto::RegisterPublicKeyCredential;

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

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFinishRequest {
    pub email: String,
    pub credential: RegisterPublicKeyCredential,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct RegisterFinishResponse {}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBeginRequest {
    pub email: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthBeginResponse {
    pub options: PublicKeyCredentialRequestOptions,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AssertionResponse {
    pub credential_id: String,
    pub authenticator_data: Vec<u8>,
    pub client_data_json: Vec<u8>,
    pub signature: Vec<u8>,
    pub user_handle: Option<Vec<u8>>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthFinishRequest {
    pub email: String,
    pub assertion: AssertionResponse,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct AuthFinishResponse {
    pub session_token: String,
}


#[derive(Debug, Serialize, Deserialize)]
pub struct SaveTextRequest {
    pub description: String,
    pub message: String,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct SaveTextResponse {}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SavedItem {
    pub saved_id: i32,
    pub description: String,
    pub data_type: String,
    pub created_at: String,
    pub message: Option<String>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct ReadItemsResponse {
    pub items: Vec<SavedItem>,
}
