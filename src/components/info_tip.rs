use leptos::*;

#[component]
pub fn InfoTip(text: &'static str) -> impl IntoView {
    view! {
        <span class="info-tip" tabindex="0">
            "[?]"
            <span class="info-tip-body">{text}</span>
        </span>
    }
}
