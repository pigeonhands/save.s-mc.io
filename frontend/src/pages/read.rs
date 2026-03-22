use crate::{components, providers::api};

use common::{AssertionResponse, SavedItem};
use leptos_use::storage::use_session_storage;
use leptos::server::codee::string::FromToStringCodec;
use leptos::{prelude::*, task::spawn_local};
use wasm_bindgen_futures::JsFuture;

#[component]
pub fn Read() -> impl IntoView {
    view! {
        <div class="h-full" box-="round" shear-="top">
            <div class="header">
                <span class="box-title">
                    <h1>"Read"</h1>
                </span>
            </div>
            <div class="p-5 flex flex-col items-center h-full" gap-="0">
                <ReadPage />
            </div>
        </div>
    }
}

#[derive(Clone)]
enum PageState {
    Login,
    ShowingItems(Vec<SavedItem>),
}

#[component]
fn ReadPage() -> impl IntoView {
    let (state, set_state) = signal(PageState::Login);

    let view_state = move || match state.get() {
        PageState::Login => view! {
            <LoginForm set_state />
        }
        .into_any(),
        PageState::ShowingItems(items) => view! {
            <ItemsList items />
        }
        .into_any(),
    };

    view! {
        {view_state}
    }
}

#[component]
fn LoginForm(set_state: WriteSignal<PageState>) -> impl IntoView {
    let (session_token, set_session_token, _) =
        use_session_storage::<String, FromToStringCodec>("session_token");

    let (email, set_email) = signal(String::new());
    let (popover_text, set_popover_text) = signal(String::new());
    let (loading, set_loading) = signal(false);

    // If we already have a session token, skip to items
    Effect::new(move |_| {
        let token = session_token.get();
        if !token.is_empty() {
            set_loading.set(true);
            spawn_local(async move {
                match api::read_items(&token).await {
                    Ok(resp) => set_state.set(PageState::ShowingItems(resp.items)),
                    Err(_) => {
                        set_session_token.set(String::new());
                        set_loading.set(false);
                    }
                }
            });
        }
    });

    let on_login = move |_| {
        let email_val = email.get_untracked();
        if email_val.is_empty() {
            set_popover_text.set("Please enter your email address.".into());
            return;
        }

        set_loading.set(true);
        spawn_local(async move {
            let result = async {
                // Step 1: auth begin
                let begin_resp = api::auth_begin(email_val.clone()).await?;

                // Step 2: call navigator.credentials.get()
                let options_js = serde_wasm_bindgen::to_value(&begin_resp.options)
                    .map_err(|e| anyhow::anyhow!("serialize options failed: {e:?}"))?;
                let pub_key = web_sys::PublicKeyCredentialRequestOptions::from(options_js);
                let opts = web_sys::CredentialRequestOptions::new();
                opts.set_public_key(&pub_key);

                let cred_js =
                    JsFuture::from(
                        gloo::utils::window()
                            .navigator()
                            .credentials()
                            .get_with_options(&opts)
                            .map_err(|e| anyhow::anyhow!("credentials.get() error: {e:?}"))?,
                    )
                    .await
                    .map_err(|e| anyhow::anyhow!("credentials.get() rejected: {e:?}"))?;

                let cred = web_sys::PublicKeyCredential::from(cred_js);
                let response = web_sys::AuthenticatorAssertionResponse::from(
                    wasm_bindgen::JsValue::from(cred.response()),
                );

                let credential_id = cred.id();
                let authenticator_data =
                    js_sys::Uint8Array::new(&response.authenticator_data()).to_vec();
                let client_data_json =
                    js_sys::Uint8Array::new(&response.client_data_json()).to_vec();
                let signature = js_sys::Uint8Array::new(&response.signature()).to_vec();
                let user_handle = response
                    .user_handle()
                    .map(|buf| js_sys::Uint8Array::new(&buf).to_vec());

                let assertion = AssertionResponse {
                    credential_id,
                    authenticator_data,
                    client_data_json,
                    signature,
                    user_handle,
                };

                // Step 3: auth finish
                let finish_resp = api::auth_finish(email_val, assertion).await?;
                let token = finish_resp.session_token;

                // Step 4: fetch items
                let items_resp = api::read_items(&token).await?;

                anyhow::Ok((token, items_resp.items))
            }
            .await;

            match result {
                Ok((token, items)) => {
                    set_session_token.set(token);
                    set_state.set(PageState::ShowingItems(items));
                }
                Err(e) => {
                    set_popover_text.set(format!("Login failed: {e}"));
                    set_loading.set(false);
                }
            }
        });
    };

    view! {
        <components::popover::Popover text={popover_text} />

        <div box-="round" shear-="top" class="w-full">
            <div class="header">
                <span class="box-title">Email Address</span>
            </div>
            <div class="p-3">
                <input
                    type="email"
                    class="w-full"
                    placeholder="you@example.com"
                    prop:value={move || email.get()}
                    on:input:target={move |e| set_email.set(e.target().value())}
                />
            </div>
        </div>

        <div class="grid grid-cols-1">
            <button
                type="button"
                variant-="foreground1"
                prop:disabled={move || loading.get()}
                on:click={on_login}
            >
                {move || if loading.get() { "Authenticating..." } else { "Login with Security Key" }}
            </button>
        </div>
    }
}

#[component]
fn ItemsList(items: Vec<SavedItem>) -> impl IntoView {
    if items.is_empty() {
        return view! {
            <p class="text-center">"No saved items yet."</p>
        }
        .into_any();
    }

    view! {
        <div class="w-full space-y-2">
            {items.into_iter().map(|item| view! {
                <div box-="round" shear-="top" class="w-full">
                    <div class="header">
                        <span class="box-title">{item.description.clone()}</span>
                        <span class="text-sm opacity-60">{item.data_type.clone()}</span>
                        <span class="text-sm opacity-60">{item.created_at.clone()}</span>
                    </div>
                    {item.message.map(|msg| view! {
                        <div class="p-3">
                            <pre class="whitespace-pre-wrap break-all">{msg}</pre>
                        </div>
                    })}
                </div>
            }).collect::<Vec<_>>()}
        </div>
    }
    .into_any()
}
