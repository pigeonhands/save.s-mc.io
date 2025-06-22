use leptos::{html, prelude::*};

#[component]
pub fn Popover(text: ReadSignal<String>) -> impl IntoView {
    let floating_ref = NodeRef::<html::Div>::new();

    Effect::new(move || {
        if !text.get().is_empty() {
            if let Some(el) = floating_ref.get() {
                if let Err(e) = el.show_popover() {
                    log::error!("Failed to show popover. {:?}", e);
                }
            }
        }
    });

    view! {
        <div
            is-="popover"
            node_ref=floating_ref
            popover="auto"
            class="m-auto"
            id="info-popover"
            >
            <div box-="round" id="content" class="">
                <p>{ move || text.get() }</p>
                <div id="buttons" class="flex justify-center w-full">
                    <button
                        box-="round"
                        popovertarget="info-popover"
                        popovertargetaction="hide"
                        >Ok</button>
                </div>
            </div>
        </div>
    }
}
