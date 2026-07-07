use leptos::*;

use crate::components::calculator_form::CalculatorForm;
use crate::components::income_charts::{RateCurveChart, RatePoint, TaxPieChart};
use crate::components::results_table::ResultsTable;
use crate::domain::calculator::calculate_income;
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, first_year_repayments, load_income_context_from_saved_input,
    DebtRecycleInput, MortgagePortfolioInput,
};
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorError, CalculatorInput, IncomeUnit};
use crate::storage::{
    load_from_storage, load_raw_from_storage, save_to_storage, DEBT_RECYCLE_STORAGE_KEY,
    INCOME_STORAGE_KEY, MORTGAGE_STORAGE_KEY,
};

pub fn load_household_outgoings() -> (f64, f64) {
    let Some(raw) = load_raw_from_storage(MORTGAGE_STORAGE_KEY) else {
        return (0.0, 0.0);
    };
    let Ok(mut portfolio) = serde_json::from_str::<MortgagePortfolioInput>(&raw) else {
        return (0.0, 0.0);
    };
    portfolio.debt_recycle = load_from_storage::<DebtRecycleInput>(DEBT_RECYCLE_STORAGE_KEY);

    let dr_annual = portfolio
        .debt_recycle
        .as_ref()
        .filter(|dr| dr.enabled)
        .map(|dr| dr.monthly_redraw_aud * 12.0)
        .unwrap_or(0.0);

    let income_ctx = load_raw_from_storage(INCOME_STORAGE_KEY)
        .as_deref()
        .and_then(load_income_context_from_saved_input);
    let mortgage_annual = calculate_mortgage_portfolio(&portfolio, income_ctx.as_ref())
        .ok()
        .map(|out| first_year_repayments(&out.amortization_rows, &out.chart_series.period_months))
        .unwrap_or(0.0);

    (mortgage_annual, dr_annual)
}

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
                        let (mortgage_annual, debt_recycling_annual) = load_household_outgoings();
                        view! {
                            <TaxPieChart
                                result=result
                                mortgage_annual=mortgage_annual
                                debt_recycling_annual=debt_recycling_annual
                            />
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
