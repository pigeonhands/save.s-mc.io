use crate::{
    components,
    providers::{api, pgputils},
};

use common::PublicKeyCredentialCreationOptions;
use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn Register() -> impl IntoView {
    view! {
        <div class="h-full" box-="round" shear-="top">
            <div class="header" >
                <span class="box-title">
                    <h1> "Register" </h1>
                </span>
            </div>

            <div class="p-5 flex flex-col items-center h-full" gap-="0">
                <RegisterForm />
            </div>
        </div>
    }
}

#[derive(Debug, Clone)]
pub struct VerifyData {
    pub email: String,
    pub passkey: std::sync::Arc<PublicKeyCredentialCreationOptions>,
    pub pgp: String,
}

impl VerifyData {
    pub fn new(email: String, passkey: PublicKeyCredentialCreationOptions, pgp: String) -> Self {
        Self {
            email,
            passkey: std::sync::Arc::new(passkey),
            pgp,
        }
    }
}

#[derive(Debug, Clone)]
pub enum RegisterFormProgress {
    Empty,
    PgpKeyLoaded(pgputils::PublicKeyInfo),
    VerificationBegin(VerifyData),
    Registered,
}

#[component]
pub fn RegisterForm() -> impl IntoView {
    let (form_progress, set_form_progress) = signal(RegisterFormProgress::Empty);

    let form_flow = move || {
        let current_form_progress = form_progress.get();

        match current_form_progress {
            RegisterFormProgress::Empty => view! {
                <EnterPgpKey set_form_progress />
            }
            .into_any(),
            RegisterFormProgress::PgpKeyLoaded(key) => view! {
                <SubmitPgpKey set_form_progress key />
            }
            .into_any(),
            RegisterFormProgress::VerificationBegin(data) => view! {
                <Verify data set_form_progress />
            }
            .into_any(),
            RegisterFormProgress::Registered => view! {
                <div class="text-center">
                    <p>"Registration complete! You can now log in."</p>
                </div>
            }
            .into_any(),
        }
    };

    view! {
       {form_flow}
    }
}

#[component]
pub fn EnterPgpKey(set_form_progress: WriteSignal<RegisterFormProgress>) -> impl IntoView {
    let is_validating = RwSignal::new(false);
    let (pgp_key_armor, set_pgp_key_armor) = signal(String::new());

    let (popover_text, set_popover_text) = signal(String::new());

    let on_pgp_submit = move |_| {
        set_popover_text.set("Test".into());
        is_validating.set(true);
        let selected_pgp_key = pgp_key_armor.get();

        if selected_pgp_key.is_empty() {
            return;
        }

        let key_info = match pgputils::get_public_key_info(pgp_key_armor.get()) {
            Ok(key) => key,
            Err(e) => {
                log::error!("Failed to fetch pgp info {:?}", e);
                set_popover_text.set("PGP public key is invalid".into());
                return;
            }
        };

        if key_info.emails.is_empty() {
            set_popover_text.set("PGP public key has no user ids with email addresses".into());
            return;
        }

        if key_info.encryption_keys.is_empty() {
            set_popover_text.set("PGP public key has no encryption sub-keys".into());
            return;
        }

        set_form_progress.set(RegisterFormProgress::PgpKeyLoaded(key_info));
    };

    let is_valid = move |input: ReadSignal<String>| !is_validating.get() || !input.get().is_empty();

    view! {
        <components::popover::Popover text={popover_text} />

        <div box-="round" shear-="top" class="w-full h-full" class:invalid-input={move || !is_valid(pgp_key_armor)}>
            <div class="header">
                <span class="box-title">PGP Public Key</span>
            </div>
            <textarea
                    name="message"
                    class="w-full h-95/100 resize-none"
                    size-="large" placeholder="..."
                    style="background: var(--background)"
                    on:input:target={move |e| set_pgp_key_armor.set(e.target().value())}
                    required
            ></textarea>

        </div>



        <div class="grid grid-cols-1">
            <button
                variant-="foreground1"
                id="pgp-button"
                type="button"
                on:click={on_pgp_submit}
            >
                Continue
            </button>
        </div>

    }
}

#[component]
pub fn SubmitPgpKey(
    set_form_progress: WriteSignal<RegisterFormProgress>,
    key: pgputils::PublicKeyInfo,
) -> impl IntoView {
    let (popover_text, set_popover_text) = signal(String::new());

    let (verifying, set_verifying) = signal(false);

    let selected_email = RwSignal::new(key.emails[0].clone());
    let selected_encryption_key = RwSignal::new(key.encryption_keys[0].0.clone());

    let on_verify = {
        let pub_key = RwSignal::new(key.armored_key.clone());

        let begin_verification = async move || -> anyhow::Result<()> {
            let register_begin_resp = api::begin_registration(
                selected_email.get_untracked(),
                selected_encryption_key.get_untracked(),
                pub_key.get_untracked(),
            )
            .await?;

            set_form_progress.set(RegisterFormProgress::VerificationBegin(VerifyData::new(
                selected_email.get_untracked(),
                register_begin_resp.passkey_challenge,
                register_begin_resp.pgp_channenge,
            )));
            Ok(())
        };

        move |_| {
            set_verifying.set(true);
            spawn_local(async move {
                match begin_verification().await {
                    Ok(_) => {}
                    Err(e) => {
                        set_popover_text.set(format!("Failed to begin. {:?}", e.to_string()));
                    }
                }
                set_verifying.set(false);
            });
        }
    };

    view! {
        <components::popover::Popover text={popover_text} />

        <div box-="round" shear-="top" class="w-full h-full">
            <div class="header">
                <span class="box-title">Email Address</span>
            </div>

            {key.emails.iter().map(|email| view! {
                <label>
                  <input type="radio" name="email" value={email.clone()} bind:group=selected_email />
                    { email.clone() }
                </label>
            }).collect::<Vec<_>>()}

        </div>


        <div box-="round" shear-="top" class="w-full h-full">
            <div class="header">
                <span class="box-title">Encryption Key</span>
            </div>

            {key.encryption_keys.iter().map(|(fingerprint, expires_at)| {
                let expires_at_str = if let Some(expires) = expires_at {
                    let local_time : chrono::DateTime<chrono::Local> = chrono::DateTime::from(*expires);
                    format!("(expires: {})", local_time.to_rfc2822())
                }else {
                    "(never expires)".into()
                };
                view! {
                    <label>
                    <input type="radio" name="encryption-key" value={fingerprint.clone()} bind:group=selected_encryption_key />
                      <span>{fingerprint.clone()}</span><span>{expires_at_str}</span>
                    </label>
                }}).collect::<Vec<_>>()
            }

        </div>

        <div class="grid grid-cols-2">

            <button
                on:click={move |_| set_form_progress.set(RegisterFormProgress::Empty) }
                prop:disabled={move || verifying.get()}
                variant-="foreground2"
                id="cancel-button"
                type="button"
                class="m-[0_1ch]"
            >
                Cancel
            </button>
            <button
                prop:disabled={move || verifying.get()}
                type="button"
                on:click={on_verify}
                variant-="foreground1"
                id="verify-button"> {
                move || if verifying.get() {
                    "Setting up verification..."
                } else {
                    "Continue"
                }
            }
            </button>
        </div>
    }
}

#[component]
pub fn Verify(
    data: VerifyData,
    set_form_progress: WriteSignal<RegisterFormProgress>,
) -> impl IntoView {
    let (popover_text, set_popover_text) = signal(String::new());
    let (verifying, set_verifying) = signal(false);

    let on_verify = move |_| {
        let ccr = {
            let js_value = serde_wasm_bindgen::to_value(&data.passkey).unwrap();
            let pub_key = web_sys::PublicKeyCredentialCreationOptions::from(js_value);
            let opts = web_sys::CredentialCreationOptions::new();
            opts.set_public_key(&pub_key);
            opts
        };

        let create_credentials = JsFuture::from(
            gloo::utils::window()
                .navigator()
                .credentials()
                .create_with_options(&ccr)
                .unwrap(),
        );

        let email = data.email.clone();
        set_verifying.set(true);
        spawn_local(async move {
            let credential = {
                let js_value = match create_credentials.await {
                    Ok(v) => v,
                    Err(e) => {
                        log::error!("credentials.create() failed: {:?}", e);
                        set_popover_text.set("Security key registration failed. Please try again.".into());
                        set_verifying.set(false);
                        return;
                    }
                };
                webauthn_rs_proto::RegisterPublicKeyCredential::from(
                    web_sys::PublicKeyCredential::from(js_value),
                )
            };

            match api::finish_registration(email, credential).await {
                Ok(_) => {
                    set_form_progress.set(RegisterFormProgress::Registered);
                }
                Err(e) => {
                    set_popover_text.set(format!("Registration failed: {}", e));
                    set_verifying.set(false);
                }
            }
        });
    };

    view! {
        <components::popover::Popover text={popover_text} />
        <button
            type="button"
            prop:disabled={move || verifying.get()}
            on:click={on_verify}
            variant-="foreground1"
            id="verify-button">
            { move || if verifying.get() { "Registering..." } else { "Verify with Security Key" } }
        </button>
    }
}
