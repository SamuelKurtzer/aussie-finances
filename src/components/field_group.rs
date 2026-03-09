use leptos::*;

#[component]
pub fn FieldGroup(
    label: &'static str,
    #[prop(optional)] help: Option<&'static str>,
    children: Children,
) -> impl IntoView {
    view! {
        <section class="field-group">
            <h3>{label}</h3>
            {help.map(|text| view! { <p class="muted">{text}</p> })}
            {children()}
        </section>
    }
}
