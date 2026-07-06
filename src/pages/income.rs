use leptos::*;

use crate::components::calculator_form::CalculatorForm;
use crate::components::results_table::ResultsTable;
use crate::domain::calculator::calculate_income;
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorError, CalculatorInput};
use crate::storage::{load_from_storage, save_to_storage, INCOME_STORAGE_KEY};

#[component]
pub fn IncomePage() -> impl IntoView {
    let input = create_rw_signal(
        load_from_storage::<CalculatorInput>(INCOME_STORAGE_KEY).unwrap_or_default(),
    );

    create_effect(move |_| {
        let current = input.get();
        save_to_storage(INCOME_STORAGE_KEY, &current);
    });

    let computed = create_memo(move |_| {
        let current = input.get();
        let rules = TaxRules::for_year(current.financial_year);
        calculate_income(&current, &rules)
    });

    view! {
        <section class="income-layout">
            <div class="panel">
                <h2>"Income Calculator"</h2>
                <p class="muted">
                    "Gross-to-net estimate for FY 2024-25 or 2025-26. Resident, non-resident, and working holiday maker rates."
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
