use leptos::*;

use crate::components::calculator_form::CalculatorForm;
use crate::components::results_table::ResultsTable;
use crate::domain::calculator::calculate_income;
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorError, CalculatorInput};

#[cfg(target_arch = "wasm32")]
const INCOME_CALC_STORAGE_KEY: &str = "aus_fin_income_calculator_v1";

#[cfg(target_arch = "wasm32")]
fn load_saved_input() -> CalculatorInput {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(raw)) = storage.get_item(INCOME_CALC_STORAGE_KEY) {
                if let Ok(parsed) = serde_json::from_str::<CalculatorInput>(&raw) {
                    return parsed;
                }
            }
        }
    }
    CalculatorInput::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_saved_input() -> CalculatorInput {
    CalculatorInput::default()
}

#[cfg(target_arch = "wasm32")]
fn persist_input(input: &CalculatorInput) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(serialized) = serde_json::to_string(input) {
                let _ = storage.set_item(INCOME_CALC_STORAGE_KEY, &serialized);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_input(_input: &CalculatorInput) {}

#[component]
pub fn IncomePage() -> impl IntoView {
    let input = create_rw_signal(load_saved_input());
    let rules = TaxRules::fy_2025_26_resident();

    create_effect(move |_| {
        let current = input.get();
        persist_input(&current);
    });

    let computed = create_memo(move |_| calculate_income(&input.get(), &rules));

    view! {
        <section class="income-layout">
            <div class="panel">
                <h2>"Income Calculator"</h2>
                <p class="muted">
                    "Resident-only FY 2025-26 estimate for gross-to-net income."
                </p>
                <CalculatorForm input=input />
            </div>

            <div class="panel">
                {move || {
                    match computed.get() {
                        Ok(result) => view! { <ResultsTable result=result /> }.into_view(),
                        Err(CalculatorError::Validation(issues)) => {
                            view! {
                                <section>
                                    <h3>"Validation issues"</h3>
                                    <ul>
                                        {issues
                                            .iter()
                                            .map(|i| {
                                                view! { <li class="error">{format!("{}: {}", i.field, i.message)}</li> }
                                            })
                                            .collect_view()}
                                    </ul>
                                </section>
                            }
                                .into_view()
                        }
                    }
                }}
            </div>
        </section>
    }
}
