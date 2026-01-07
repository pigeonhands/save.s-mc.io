use axum::extract::FromRef;
use coset::{CborSerializable, iana};
use passkey_types::{
    ctap2::AuthenticatorData,
    webauthn::{
        AttestationConveyancePreference, AuthenticatedPublicKeyCredential,
        AuthenticatorSelectionCriteria, PublicKeyCredentialCreationOptions,
        PublicKeyCredentialParameters, PublicKeyCredentialType, PublicKeyCredentialUserEntity,
    },
};
use uuid::Uuid;
// use webauthn_rs_proto::{
//     AuthenticatorSelectionCriteria, COSEAlgorithm, PubKeyCredParams,
//     PublicKeyCredentialCreationOptions, RegisterPublicKeyCredential, RelyingParty, User,
// };
//
#[derive(serde::Serialize, serde::Deserialize, Debug)]
#[serde(rename_all = "camelCase")]
struct CollectedClientData {
    #[serde(rename = "type")]
    pub_type: String,
    pub challenge: String, // base64
    pub origin: String,
    pub cross_origin: Option<bool>,
}

#[derive(serde::Deserialize)]
struct RawAttestationObject {
    #[serde(with = "serde_bytes")]
    #[serde(rename = "authData")]
    auth_data: Vec<u8>,
    // fmt and attStmt can be ignored for basic Passkey support
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
        response: AuthenticatedPublicKeyCredential,
        stored_challenge: &str,
        stored_rp_id: &[u8],
    ) -> anyhow::Result<Vec<u8>> {
        let client_data: CollectedClientData =
            serde_json::from_slice(&response.response.client_data_json)?;

        if client_data.challenge != stored_challenge {
            anyhow::bail!("Provieded channenge does not match store challenge");
        }

        let attestation: RawAttestationObject = ciborium::from_reader(
            &response
                .response
                .attestation_object
                .ok_or_else(|| anyhow::anyhow!("No attestation_object object"))?[..],
        )
        .map_err(|_| anyhow::anyhow!("Failed to decode attestation CBOR"))?;

        let auth_data = AuthenticatorData::from_slice(attestation.auth_data.as_slice())
            .map_err(|_| anyhow::anyhow!("Invalid AuthenticatorData format"))?;

        if auth_data.rp_id_hash() != stored_rp_id {
            anyhow::bail!("RP ID Hash mismatch. Credential not for this domain.")
        }

        let cred_data = auth_data
            .attested_credential_data
            .ok_or_else(|| anyhow::anyhow!("No credential data found in attestation"))?;

        Ok(cred_data
            .key
            .to_vec()
            .map_err(|e| anyhow::anyhow!("Error convering cred key to vec: {}", e))?)
    }
}
