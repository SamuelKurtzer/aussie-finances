use leptos::*;

#[component]
pub fn InfoTip(text: &'static str) -> impl IntoView {
    view! {
        // prevent_default so a tip inside a <summary> doesn't toggle the section
        <span class="info-tip" tabindex="0" on:click=|ev| ev.prevent_default()>
            "[?]"
            <span class="info-tip-body">{text}</span>
        </span>
    }
}
