use leptos::{
    prelude::*,
    reactive::spawn_local,
    server::codee::string::{FromToStringCodec, OptionCodec},
};
use leptos_use::storage::use_session_storage;

use crate::providers::api;

#[component]
pub fn Save() -> impl IntoView {
    view! {
        <ErrorBoundary fallback=|errors| {
            view! {
                <h1>"Uh oh! Something went wrong!"</h1>

                <p>"Errors: "</p>
                // Render a list of errors as strings - good for development purposes
                <ul>
                    {move || {
                        errors
                            .get()
                            .into_iter()
                            .map(|(_, e)| view! { <li>{e.to_string()}</li> })
                            .collect_view()
                    }}

                </ul>
            }
        }>
            <div class="h-full" box-="round" shear-="top">
                <div class="header" >
                    <span class="box-title">
                        <h1> "Save" </h1>
                    </span>
                </div>

                <SaveForm />
            </div>
        </ErrorBoundary >
    }
}

#[component]
pub fn SaveForm() -> impl IntoView {
    let (last_email_saved, set_last_email_saved, _) =
        use_session_storage::<String, FromToStringCodec>(format!("settings:last-email"));

    let (email, set_email) = signal(last_email_saved.get_untracked());
    let (description, set_description) = signal(String::new());
    let (message, set_message) = signal(String::new());
    let (encrypted_message, set_encrypted_message) = signal(Option::<String>::None);

    let (message_display, set_message_display) = signal(String::new());

    let (encrypting, set_encrypting) = signal(false);
    let is_validating = RwSignal::new(false);
    let clear_cache = RwSignal::new(false);

    Effect::new(move |_| {
        if let Some(enc_msg) = encrypted_message.get() {
            set_message_display.set(enc_msg);
        } else {
            set_message_display.set(message.get());
        }
    });

    let encrypt_message =
        |encryption_key: String,
         description: ReadSignal<String>,
         message: ReadSignal<String>,
         set_encrypted_message: WriteSignal<Option<String>>| {
            match crate::providers::pgp::encrypt(
                encryption_key,
                description.get_untracked(),
                message.get_untracked(),
            ) {
                Ok(msg) => {
                    set_encrypted_message.set(Some(msg));
                }
                Err(e) => {
                    log::error!("Failed to encrypt message. {:?}", e);
                }
            }
        };

    let on_encrypt = move |_| {
        let email = email;
        let message = message;
        let description = description;
        let set_encrypted_message = set_encrypted_message;

        is_validating.set(true);

        if email.get().is_empty() || description.get().is_empty() || message.get().is_empty() {
            log::warn!("tried to encrypt with incomplete form");
            return;
        }

        set_encrypting.set(true);
        set_last_email_saved.set(email.get());

        let (gpg_key, set_gpg_key, delete_gpg_cache) = use_session_storage::<
            Option<String>,
            OptionCodec<FromToStringCodec>,
        >(format!("pgp:{}", email.get()));

        let cached_key = if clear_cache.get() {
            log::warn!("Clearning gpg cache for {}", email.get());
            delete_gpg_cache();
            None
        } else {
            gpg_key.get()
        };

        log::debug!("GPG key in storage: {:?}", cached_key);

        if let Some(key) = cached_key {
            encrypt_message(key, description, message, set_encrypted_message);

            set_encrypting.set(false);
        } else {
            spawn_local(async move {
                let new_key = api::get_public_key(email.get_untracked()).await;

                match new_key {
                    Ok(key) => {
                        log::info!("Got new pub key for {}", key.email);
                        set_gpg_key.set(Some(key.pub_key.clone()));

                        encrypt_message(key.pub_key, description, message, set_encrypted_message);
                    }
                    Err(e) => {
                        log::error!("Failed to get pub_key. {:?}", e);
                    }
                };

                set_encrypting.set(false);
            });
        }
    };

    let on_save = move |_| {
        //
        // TODO
    };

    let is_valid = move |input: ReadSignal<String>| !is_validating.get() || !input.get().is_empty();

    view! {
            <div class="p-5 flex flex-col items-center h-full" gap-="0">

                <div class="grid grid-cols-2 justify-between">
                    <div box-="round" shear-="top" class:invalid-input={move || !is_valid(email)}>
                        <span class="box-title">Email</span>
                        <input
                            name="email"
                            class="h-11 w-full"
                            style="background: var(--background)"
                            on:input:target={move |e| set_email.set(e.target().value())}
                            prop:disabled={move || encrypted_message.get().is_some()}
                            value={move || last_email_saved.get()}
                            required
                        />
                    </div>

                    <div box-="round" shear-="top" class:invalid-input={move || !is_valid(description)}>
                        <span class="box-title">Description</span>
                        <input
                            name="email"
                            class="h-11 w-full"
                            style="background: var(--background)"
                            on:input:target={move |e| set_description.set(e.target().value())}
                            prop:disabled={move || encrypted_message.get().is_some()}
                            required
                        />
                    </div>
                </div>

                <div box-="round" shear-="top" class="w-full h-full" class:invalid-input={move || !is_valid(message)}>
                    <div class="header">
                        <span class="box-title">Message</span>
                        <span>
                            <span is-="badge" class="mr-[2ch]">Text</span>
                            <span is-="badge" variant-="background1" class="mr-[2ch]">File</span>
                        </span>
                    </div>
                    <textarea
                            prop:value={message_display}
                            on:input:target={move |e| set_message.set(e.target().value())}
                            //disabled={encrypting || encrypted}
                            name="message"
                            class="w-full h-95/100 resize-none"
                            size-="large" placeholder="..."
                            style="background: var(--background)"
                            prop:disabled={move || encrypted_message.get().is_some()}
                            required
                    ></textarea>

                </div>


            { move || if encrypted_message.get().is_some() {
                view! {

                <div class="grid grid-cols-2">
                    <button
                        on:click={move |_| set_encrypted_message.set(None)}
                        variant-="foreground2"
                        id="login-button"
                        type="button"
                        class="m-[0_1ch]"
                    >
                        Edit message
                    </button>

                    <button
                        on:click={on_save}
                        variant-="foreground1"
                        id="login-button"
                        type="button"
                        class="m-[0_1ch]"
                    >
                        Save
                    </button>

                </div>
                }.into_any()
                } else {
                    view!{
                    <div class="grid grid-rows-1">
                        <button
                            prop:disabled={move || encrypting.get()}
                            type="button"
                            on:click={on_encrypt}
                            variant-="foreground1"
                            id="login-button"> {
                            move || if encrypting.get() {
                                "Encrypting..."
                            } else {
                                "Encrypt"
                            }
                        }
                        </button>
                        // <label>
                        //     <input type="checkbox" bind:checked=clear_cache />
                        //     Clear gpg pub key cache
                        // </label>
                    </div>
                    }.into_any()
                }}

                </div>

    }
}
