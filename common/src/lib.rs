use serde::{Deserialize, Serialize};

#[derive(Serialize, Deserialize)]
pub struct PublicKeyRequest {
    pub email: String,
}

impl PublicKeyRequest {
    pub fn iter(&self) -> impl Iterator<Item = (&'static str, String)> {
        [("email", self.email.clone())].into_iter()
    }
}

#[derive(Serialize, Deserialize)]
pub struct PublicKeyResponse {
    pub email: String,
    pub pub_key: String,
}
