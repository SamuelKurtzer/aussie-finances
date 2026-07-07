use leptos::*;

use crate::components::calculator_form::CalculatorForm;
use crate::components::income_charts::{RateCurveChart, RatePoint, TaxPieChart};
use crate::components::results_table::ResultsTable;
use crate::domain::calculator::calculate_income;
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorError, CalculatorInput, IncomeUnit};
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

    let rate_curve = create_memo(move |_| {
        let current = input.get();
        let rules = TaxRules::for_year(current.financial_year);
        let base_gross = match calculate_income(&current, &rules) {
            Ok(out) => out.gross_income_annual,
            Err(_) => return Vec::new(),
        };
        let x_max = (base_gross * 2.0).max(200_000.0);
        let steps = 120;
        let mut sweep = current.clone();
        sweep.income_unit = IncomeUnit::Annual;
        sweep.includes_super = false;
        sweep.bonus_annual = 0.0;
        sweep.overtime_annual = 0.0;
        (0..=steps)
            .filter_map(|i| {
                sweep.income_amount = x_max * i as f64 / steps as f64;
                calculate_income(&sweep, &rules).ok().map(|out| RatePoint {
                    gross: out.gross_income_annual,
                    effective: out.effective_tax_rate_percent,
                    marginal: out.marginal_rate_percent,
                })
            })
            .collect::<Vec<_>>()
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

                {move || {
                    computed.get().ok().map(|result| {
                        let points = rate_curve.get();
                        let current_gross = result.gross_income_annual;
                        let current_effective = result.effective_tax_rate_percent;
                        let current_marginal = result.marginal_rate_percent;
                        view! {
                            <TaxPieChart result=result />
                            {(points.len() >= 2).then(|| view! {
                                <RateCurveChart
                                    points=points.clone()
                                    current_gross=current_gross
                                    current_effective=current_effective
                                    current_marginal=current_marginal
                                />
                            })}
                        }
                    })
                }}
            </div>
        </section>
    }
}
