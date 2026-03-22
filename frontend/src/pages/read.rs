use crate::{components, providers::api};

use common::{AssertionResponse, SavedItem};
use leptos::server::codee::string::FromToStringCodec;
use leptos::{prelude::*, task::spawn_local};
use leptos_use::storage::use_session_storage;
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
    ShowingItems,
}

#[component]
fn ReadPage() -> impl IntoView {
    let (session_token, set_session_token, _) =
        use_session_storage::<String, FromToStringCodec>("session_token");
    let items = RwSignal::new(Vec::<SavedItem>::new());
    let (state, set_state) = signal(PageState::Login);

    let view_state = move || match state.get() {
        PageState::Login => view! {
            <LoginForm set_state set_session_token items />
        }
        .into_any(),
        PageState::ShowingItems => view! {
            <ItemsList items session_token />
        }
        .into_any(),
    };

    view! { {view_state} }
}

#[component]
fn LoginForm(
    set_state: WriteSignal<PageState>,
    set_session_token: WriteSignal<String>,
    items: RwSignal<Vec<SavedItem>>,
) -> impl IntoView {
    let (session_token, _, _) =
        use_session_storage::<String, FromToStringCodec>("session_token");

    let (popover_text, set_popover_text) = signal(String::new());
    let (loading, set_loading) = signal(false);

    Effect::new(move |_| {
        let token = session_token.get();
        if !token.is_empty() {
            set_loading.set(true);
            spawn_local(async move {
                match api::read_items(&token).await {
                    Ok(resp) => {
                        items.set(resp.items);
                        set_state.set(PageState::ShowingItems);
                    }
                    Err(_) => {
                        set_session_token.set(String::new());
                        set_loading.set(false);
                    }
                }
            });
        }
    });

    let on_login = move |_| {
        set_loading.set(true);
        spawn_local(async move {
            let result = async {
                let begin_resp = api::auth_begin().await?;

                let options_js = serde_wasm_bindgen::to_value(&begin_resp.options)
                    .map_err(|e| anyhow::anyhow!("serialize options failed: {e:?}"))?;
                let pub_key = web_sys::PublicKeyCredentialRequestOptions::from(options_js);
                let opts = web_sys::CredentialRequestOptions::new();
                opts.set_public_key(&pub_key);

                let cred_js = JsFuture::from(
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

                let assertion = AssertionResponse {
                    credential_id: cred.id(),
                    authenticator_data: js_sys::Uint8Array::new(&response.authenticator_data()).to_vec(),
                    client_data_json: js_sys::Uint8Array::new(&response.client_data_json()).to_vec(),
                    signature: js_sys::Uint8Array::new(&response.signature()).to_vec(),
                    user_handle: response.user_handle().map(|buf| js_sys::Uint8Array::new(&buf).to_vec()),
                };

                let finish_resp = api::auth_finish(begin_resp.challenge_id, assertion).await?;
                let token = finish_resp.session_token;
                let items_resp = api::read_items(&token).await?;

                anyhow::Ok((token, items_resp.items))
            }
            .await;

            match result {
                Ok((token, loaded_items)) => {
                    set_session_token.set(token);
                    items.set(loaded_items);
                    set_state.set(PageState::ShowingItems);
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
fn ItemsList(items: RwSignal<Vec<SavedItem>>, session_token: Signal<String>) -> impl IntoView {
    let (popover_text, set_popover_text) = signal(String::new());

    view! {
        <components::popover::Popover text={popover_text} />
        <div class="w-full space-y-2">
            <Show when=move || items.get().is_empty()>
                <p class="text-center">"No saved items yet."</p>
            </Show>
            <For
                each=move || items.get()
                key=|item| item.saved_id.clone()
                children=move |item| {
                    let saved_id = item.saved_id.clone();
                    let on_delete = move |_| {
                        if !gloo::utils::window().confirm_with_message("Delete this item?").unwrap_or(false) {
                            return;
                        }
                        let token = session_token.get_untracked();
                        let id = saved_id.clone();
                        spawn_local(async move {
                            match api::delete_item(&token, &id).await {
                                Ok(_) => items.update(|v| v.retain(|i| i.saved_id != id)),
                                Err(e) => set_popover_text.set(format!("Delete failed: {e}")),
                            }
                        });
                    };
                    view! {
                        <div box-="round" shear-="top" class="w-full">
                            <div class="header">
                                <span class="box-title">{item.description.clone()}</span>
                                <span class="text-sm opacity-60">{item.created_at.clone()}</span>
                                <span is-="badge" variant-="foreground2" on:click={on_delete} style="cursor:pointer">
                                    "Delete"
                                </span>
                            </div>
                            {item.message.map(|msg| view! {
                                <div class="p-3">
                                    <pre class="whitespace-pre-wrap break-all">{msg}</pre>
                                </div>
                            })}
                        </div>
                    }
                }
            />
        </div>
    }
}
