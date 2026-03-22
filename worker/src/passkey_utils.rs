use axum::extract::FromRef;
use base64::Engine;
use coset::{CborSerializable, iana};
use passkey_types::{
    ctap2::AuthenticatorData,
    webauthn::{
        AttestationConveyancePreference, AuthenticatorSelectionCriteria,
        PublicKeyCredentialCreationOptions, PublicKeyCredentialDescriptor,
        PublicKeyCredentialParameters, PublicKeyCredentialRequestOptions,
        PublicKeyCredentialType, PublicKeyCredentialUserEntity,
        UserVerificationRequirement,
    },
};
use uuid::Uuid;

#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CollectedClientData {
    #[serde(rename = "type")]
    pub_type: String,
    pub challenge: String,
    pub origin: String,
    pub cross_origin: Option<bool>,
}

#[derive(serde::Deserialize)]
struct RawAttestationObject {
    #[serde(with = "serde_bytes")]
    #[serde(rename = "authData")]
    auth_data: Vec<u8>,
}

use crate::http::context::AppState;
pub struct PasskeyCtx {
    host: String,
}

impl FromRef<AppState> for PasskeyCtx {
    fn from_ref(input: &AppState) -> Self {
        Self {
            host: input.uri.host().unwrap_or("localhost").into(),
        }
    }
}

impl PasskeyCtx {
    pub fn rp_id(&self) -> &str {
        &self.host
    }

    pub fn register_start(
        &self,
        email: &str,
    ) -> anyhow::Result<(PublicKeyCredentialCreationOptions, [u8; 32])> {
        let mut challenge = [0u8; 32];
        getrandom::fill(&mut challenge)
            .map_err(|e| anyhow::anyhow!("error creating passkey challenge: {:?}", e))?;

        let user_id = Uuid::new_v4();

        let rp = passkey_types::webauthn::PublicKeyCredentialRpEntity {
            name: email.into(),
            id: Some(self.host.clone()),
        };

        let user = PublicKeyCredentialUserEntity {
            id: user_id.as_bytes().to_vec().into(),
            name: email.into(),
            display_name: email.into(),
        };

        let options = PublicKeyCredentialCreationOptions {
            rp,
            user,
            challenge: challenge.to_vec().into(),
            pub_key_cred_params: vec![
                PublicKeyCredentialParameters {
                    ty: PublicKeyCredentialType::PublicKey,
                    alg: iana::Algorithm::ES256,
                },
                PublicKeyCredentialParameters {
                    ty: PublicKeyCredentialType::PublicKey,
                    alg: iana::Algorithm::EdDSA,
                },
                PublicKeyCredentialParameters {
                    ty: PublicKeyCredentialType::PublicKey,
                    alg: iana::Algorithm::RS256,
                },
            ],
            timeout: Some(60000),
            exclude_credentials: None,
            authenticator_selection: Some(AuthenticatorSelectionCriteria::default()),
            attestation: AttestationConveyancePreference::None,
            extensions: None,
            hints: None,
            attestation_formats: None,
        };

        Ok((options, challenge))
    }

    pub fn register_finish(
        response: &webauthn_rs_proto::RegisterPublicKeyCredential,
        stored_challenge_b64: &str,
        rp_id: &str,
    ) -> anyhow::Result<Vec<u8>> {
        use sha2::Digest;

        let client_data: CollectedClientData =
            serde_json::from_slice(&response.response.client_data_json)?;

        if client_data.challenge != stored_challenge_b64 {
            anyhow::bail!("Challenge does not match stored challenge");
        }

        let attestation: RawAttestationObject =
            ciborium::from_reader(&response.response.attestation_object[..])
                .map_err(|_| anyhow::anyhow!("Failed to decode attestation CBOR"))?;

        let auth_data = AuthenticatorData::from_slice(attestation.auth_data.as_slice())
            .map_err(|_| anyhow::anyhow!("Invalid AuthenticatorData format"))?;

        let expected_rp_id_hash = sha2::Sha256::digest(rp_id.as_bytes());
        if auth_data.rp_id_hash() != expected_rp_id_hash.as_slice() {
            anyhow::bail!("RP ID Hash mismatch. Credential not for this domain.");
        }

        let cred_data = auth_data
            .attested_credential_data
            .ok_or_else(|| anyhow::anyhow!("No credential data found in attestation"))?;

        Ok(cred_data
            .key
            .to_vec()
            .map_err(|e| anyhow::anyhow!("Error converting cred key to vec: {}", e))?)
    }

    pub fn auth_start(
        &self,
        credential_ids: Vec<String>,
    ) -> anyhow::Result<(PublicKeyCredentialRequestOptions, [u8; 32])> {
        let mut challenge = [0u8; 32];
        getrandom::fill(&mut challenge)
            .map_err(|e| anyhow::anyhow!("error creating auth challenge: {:?}", e))?;

        let allow_credentials = credential_ids
            .into_iter()
            .filter_map(|id| {
                base64::engine::general_purpose::URL_SAFE_NO_PAD
                    .decode(&id)
                    .ok()
                    .map(|bytes| PublicKeyCredentialDescriptor {
                        ty: PublicKeyCredentialType::PublicKey,
                        id: bytes.into(),
                        transports: None,
                    })
            })
            .collect::<Vec<_>>();

        let options = PublicKeyCredentialRequestOptions {
            challenge: challenge.to_vec().into(),
            timeout: Some(60000),
            rp_id: Some(self.host.clone()),
            allow_credentials: if allow_credentials.is_empty() {
                None
            } else {
                Some(allow_credentials)
            },
            user_verification: UserVerificationRequirement::Preferred,
            hints: None,
            attestation: AttestationConveyancePreference::None,
            attestation_formats: None,
            extensions: None,
        };

        Ok((options, challenge))
    }

    pub fn auth_finish(
        assertion: &common::AssertionResponse,
        stored_challenge_b64: &str,
        rp_id: &str,
        cose_key_b64: &str,
    ) -> anyhow::Result<()> {
        use p256::ecdsa::signature::Verifier;
        use sha2::Digest;

        // Verify client data
        let client_data: CollectedClientData =
            serde_json::from_slice(&assertion.client_data_json)
                .map_err(|e| anyhow::anyhow!("Invalid clientDataJSON: {e}"))?;

        if client_data.pub_type != "webauthn.get" {
            anyhow::bail!("Wrong client data type: {}", client_data.pub_type);
        }
        if client_data.challenge != stored_challenge_b64 {
            anyhow::bail!("Challenge mismatch");
        }

        // Verify RP ID hash in authenticatorData
        let auth_data = AuthenticatorData::from_slice(&assertion.authenticator_data)
            .map_err(|_| anyhow::anyhow!("Invalid authenticatorData"))?;

        let expected_rp_id_hash = sha2::Sha256::digest(rp_id.as_bytes());
        if auth_data.rp_id_hash() != expected_rp_id_hash.as_slice() {
            anyhow::bail!("RP ID hash mismatch");
        }

        // Decode stored COSE public key
        let cose_key_bytes = base64::engine::general_purpose::URL_SAFE_NO_PAD
            .decode(cose_key_b64)
            .map_err(|e| anyhow::anyhow!("Invalid base64 COSE key: {e}"))?;

        let cose_key = coset::CoseKey::from_slice(&cose_key_bytes)
            .map_err(|e| anyhow::anyhow!("Invalid COSE key: {e}"))?;

        // Extract x,y from EC2 COSE key
        let get_bytes = |label: i64| -> anyhow::Result<Vec<u8>> {
            cose_key
                .params
                .iter()
                .find(|(l, _)| *l == coset::Label::Int(label))
                .and_then(|(_, v)| {
                    if let ciborium::value::Value::Bytes(b) = v {
                        Some(b.clone())
                    } else {
                        None
                    }
                })
                .ok_or_else(|| anyhow::anyhow!("COSE key missing param {label}"))
        };

        let x = get_bytes(-2)?;
        let y = get_bytes(-3)?;

        // Build uncompressed SEC1 point (0x04 || x || y) and verify signature
        let mut sec1 = vec![0x04u8];
        sec1.extend_from_slice(&x);
        sec1.extend_from_slice(&y);

        let verifying_key = p256::ecdsa::VerifyingKey::from_sec1_bytes(&sec1)
            .map_err(|e| anyhow::anyhow!("Invalid EC public key: {e}"))?;

        let signature = p256::ecdsa::Signature::from_der(&assertion.signature)
            .map_err(|e| anyhow::anyhow!("Invalid DER signature: {e}"))?;

        // verificationData = authenticatorData || SHA256(clientDataJSON)
        let client_data_hash = sha2::Sha256::digest(&assertion.client_data_json);
        let mut verification_data = assertion.authenticator_data.clone();
        verification_data.extend_from_slice(&client_data_hash);

        verifying_key
            .verify(&verification_data, &signature)
            .map_err(|e| anyhow::anyhow!("Signature verification failed: {e}"))
    }
}
