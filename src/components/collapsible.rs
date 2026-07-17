use leptos::*;
use wasm_bindgen::JsCast;

use crate::storage::{load_from_storage, save_to_storage, COLLAPSE_KEY_PREFIX};

fn storage_key(title: &str) -> String {
    let slug: String = title
        .to_lowercase()
        .chars()
        .map(|c| if c.is_ascii_alphanumeric() { c } else { '_' })
        .collect();
    format!("{COLLAPSE_KEY_PREFIX}{slug}_v1")
}

/// Section with a toggleable body. Open/closed state is kept in local
/// storage (keyed by title) because result panels are re-created on every
/// input change, which would otherwise reset a plain `<details>`.
#[component]
pub fn Collapsible(
    #[prop(into)] title: String,
    #[prop(optional)] closed: bool,
    #[prop(optional)] class: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    let key = storage_key(&title);
    let open = load_from_storage::<bool>(&key).unwrap_or(!closed);
    let on_toggle = move |ev: leptos::ev::Event| {
        if let Some(details) = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::HtmlDetailsElement>().ok())
        {
            save_to_storage(&key, &details.open());
        }
    };
    let classes = match class {
        Some(extra) => format!("collapsible {extra}"),
        None => "collapsible".to_string(),
    };
    view! {
        <details class=classes open=open on:toggle=on_toggle>
            <summary><h3>{title}</h3></summary>
            {children()}
        </details>
    }
}
