use axum::{extract::FromRef, http::Uri};
use coset::iana;
use passkey_authenticator::{
    Authenticator, CredentialStore, StoreInfo, UserCheck, UserValidationMethod,
};
use passkey_client::{Client, DefaultClientData};
use passkey_types::{
    Bytes, Passkey,
    ctap2::{
        Aaguid, Ctap2Error, StatusCode,
        get_assertion::{self, Options},
        make_credential,
    },
    rand,
    webauthn::{
        AttestationConveyancePreference, AuthenticatedPublicKeyCredential,
        CreatedPublicKeyCredential, CredentialCreationOptions, CredentialRequestOptions,
        PublicKeyCredentialCreationOptions, PublicKeyCredentialDescriptor,
        PublicKeyCredentialParameters, PublicKeyCredentialRequestOptions,
        PublicKeyCredentialRpEntity, PublicKeyCredentialType, PublicKeyCredentialUserEntity,
        UserVerificationRequirement,
    },
};
use webauthn_rp::request::{AsciiDomain, RpId, register::UserHandle64};
use worker::Url;

use crate::http::context::AppState;

// MyUserValidationMethod is a stub impl of the UserValidationMethod trait, used later.
struct MyUserValidationMethod {}
#[async_trait::async_trait]
impl UserValidationMethod for MyUserValidationMethod {
    type PasskeyItem = Passkey;

    async fn check_user<'a>(
        &self,
        _credential: Option<&'a Passkey>,
        presence: bool,
        verification: bool,
    ) -> Result<UserCheck, Ctap2Error> {
        Ok(UserCheck {
            presence,
            verification,
        })
    }

    fn is_verification_enabled(&self) -> Option<bool> {
        Some(true)
    }

    fn is_presence_enabled(&self) -> bool {
        true
    }
}

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
    pub fn start_register(&self, email: &str) -> anyhow::Result<CredentialCreationOptions> {
        let user_id = UserHandle64::new();
        let rp_id = RpId::Domain(
            AsciiDomain::try_from(
                self.uri
                    .host()
                    .map(str::to_string)
                    .unwrap_or_else(|| "".into()),
            )
            .unwrap(),
        );
        todo!()
    }
}

#[derive(Debug, Default)]
struct CredStore(Option<Passkey>);

#[async_trait::async_trait]
impl CredentialStore for CredStore {
    type PasskeyItem = Passkey;

    async fn find_credentials(
        &self,
        ids: Option<&[PublicKeyCredentialDescriptor]>,
        rp_id: &str,
    ) -> Result<Vec<Self::PasskeyItem>, StatusCode> {
        log::debug!("find_credentials - id: {:?}, rp_id - {:?}", ids, rp_id);
        self.0.find_credentials(ids, rp_id).await
    }

    async fn save_credential(
        &mut self,
        cred: Passkey,
        user: passkey_types::ctap2::make_credential::PublicKeyCredentialUserEntity,
        rp: passkey_types::ctap2::make_credential::PublicKeyCredentialRpEntity,
        options: Options,
    ) -> Result<(), StatusCode> {
        log::debug!("save_credentials {:?}", cred.rp_id);
        self.0.save_credential(cred, user, rp, options).await
    }

    async fn update_credential(&mut self, cred: Passkey) -> Result<(), StatusCode> {
        log::info!("update credentials");
        self.0.update_credential(cred).await
    }

    async fn get_info(&self) -> StoreInfo {
        log::info!("get_info");
        self.0.get_info().await
    }
}

pub fn register_start(email: String) {
    let user_entity = PublicKeyCredentialUserEntity {
        id: rand::random_vec(32).into(),
        display_name: "Johnny Passkey".into(),
        name: "jpasskey@example.org".into(),
    };
    let parameters_from_rp = PublicKeyCredentialParameters {
        ty: PublicKeyCredentialType::PublicKey,
        alg: iana::Algorithm::ES256,
    };
}

// Example of how to set up, register and authenticate with a `Client`.
pub async fn register_passkey(
    origin: &Url,
    challenge_bytes_from_rp: Bytes,
) -> anyhow::Result<(CreatedPublicKeyCredential, AuthenticatedPublicKeyCredential)> {
    let user_entity = PublicKeyCredentialUserEntity {
        id: rand::random_vec(32).into(),
        display_name: "Johnny Passkey".into(),
        name: "jpasskey@example.org".into(),
    };
    let parameters_from_rp = PublicKeyCredentialParameters {
        ty: PublicKeyCredentialType::PublicKey,
        alg: iana::Algorithm::ES256,
    };
    // First create an Authenticator for the Client to use.
    let my_aaguid = Aaguid::new_empty();
    let user_validation_method = MyUserValidationMethod {};
    // Create the CredentialStore for the Authenticator.
    // Option<Passkey> is the simplest possible implementation of CredentialStore
    let store = CredStore::default();
    let my_authenticator = Authenticator::new(my_aaguid, store, user_validation_method);

    // Create the Client
    // If you are creating credentials, you need to declare the Client as mut
    let mut my_client = Client::new(my_authenticator).allows_insecure_localhost(true);

    // The following values, provided as parameters to this function would usually be
    // retrieved from a Relying Party according to the context of the application.
    let request = CredentialCreationOptions {
        public_key: PublicKeyCredentialCreationOptions {
            rp: PublicKeyCredentialRpEntity {
                id: None, // Leaving the ID as None means use the effective domain
                name: origin.host().unwrap().to_string(),
            },
            user: user_entity,
            challenge: challenge_bytes_from_rp,
            pub_key_cred_params: vec![parameters_from_rp],
            timeout: None,
            exclude_credentials: None,
            authenticator_selection: None,
            hints: None,
            attestation: AttestationConveyancePreference::None,
            attestation_formats: None,
            extensions: None,
        },
    };

    log::info!("Registering...");
    // Now create the credential.
    let my_webauthn_credential = my_client
        .register(origin, request, DefaultClientData)
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    // Let's try and authenticate.
    // Create a challenge that would usually come from the RP.
    let challenge_bytes_from_rp: Bytes = rand::random_vec(32).into();
    // Now try and authenticate
    let credential_request = CredentialRequestOptions {
        public_key: PublicKeyCredentialRequestOptions {
            challenge: challenge_bytes_from_rp,
            timeout: None,
            rp_id: None, //Some(String::from(origin.domain().unwrap())),
            allow_credentials: Some(vec![]),
            user_verification: UserVerificationRequirement::default(),
            hints: None,
            attestation: AttestationConveyancePreference::None,
            attestation_formats: None,
            extensions: None,
        },
    };

    log::info!("Authenticating...");

    let authenticated_cred = my_client
        .authenticate(origin, credential_request, DefaultClientData)
        .await
        .map_err(|e| anyhow::anyhow!("{:?}", e))?;

    Ok((my_webauthn_credential, authenticated_cred))
}

async fn authenticator_setup(
    user_entity: PublicKeyCredentialUserEntity,
    client_data_hash: Bytes,
    algorithms_from_rp: PublicKeyCredentialParameters,
    rp_id: String,
) -> Result<get_assertion::Response, StatusCode> {
    let store: Option<Passkey> = None;
    let user_validation_method = MyUserValidationMethod {};
    let my_aaguid = Aaguid::new_empty();

    let mut my_authenticator = Authenticator::new(my_aaguid, store, user_validation_method);

    let reg_request = make_credential::Request {
        client_data_hash: client_data_hash.clone(),
        rp: make_credential::PublicKeyCredentialRpEntity {
            id: rp_id.clone(),
            name: None,
        },
        user: user_entity,
        pub_key_cred_params: vec![algorithms_from_rp],
        exclude_list: None,
        extensions: None,
        options: make_credential::Options::default(),
        pin_auth: None,
        pin_protocol: None,
    };

    let credential: make_credential::Response =
        my_authenticator.make_credential(reg_request).await?;

    //ctap2_creation_success(credential);

    let auth_request = get_assertion::Request {
        rp_id,
        client_data_hash,
        allow_list: None,
        extensions: None,
        options: make_credential::Options::default(),
        pin_auth: None,
        pin_protocol: None,
    };

    let response = my_authenticator.get_assertion(auth_request).await?;

    Ok(response)
}
