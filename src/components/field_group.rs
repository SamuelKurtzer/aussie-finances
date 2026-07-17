use leptos::*;

use crate::components::collapsible::Collapsible;

#[component]
pub fn FieldGroup(
    label: &'static str,
    #[prop(optional)] help: Option<&'static str>,
    #[prop(optional)] closed: bool,
    children: Children,
) -> impl IntoView {
    view! {
        <Collapsible title=label class="field-group" closed=closed>
            {help.map(|text| view! { <p class="muted">{text}</p> })}
            {children()}
        </Collapsible>
    }
}
