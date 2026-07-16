use leptos::*;

use crate::components::calculator_form::CalculatorForm;
use crate::components::collapsible::Collapsible;
use crate::components::income_charts::{RateCurveChart, RatePoint, TaxPieChart};
use crate::components::results_table::ResultsTable;
use crate::domain::budget::BudgetInput;
use crate::domain::calculator::{calculate_income, solve_gross_for_net};
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, first_year_repayments, load_income_context_from_saved_input,
    DebtRecycleInput, MortgagePortfolioInput,
};
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorError, CalculatorInput, IncomeUnit, PayFrequency};
use crate::formatting::fmt_money;
use crate::storage::{
    load_from_storage, load_raw_from_storage, save_to_storage, BUDGET_STORAGE_KEY,
    DEBT_RECYCLE_STORAGE_KEY, INCOME_STORAGE_KEY, MORTGAGE_STORAGE_KEY,
};

fn load_budget_annual() -> f64 {
    load_from_storage::<BudgetInput>(BUDGET_STORAGE_KEY)
        .map(|b| b.annual_total())
        .unwrap_or(0.0)
}

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
        .map(|dr| dr.redraw_amount_aud * (12.0 / dr.redraw_cadence.interval_months()))
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

    let target_net = create_rw_signal(0.0_f64);
    let target_freq = create_rw_signal(PayFrequency::Annually);
    let required_gross = create_memo(move |_| {
        let target = target_net.get();
        if target <= 0.0 {
            return None;
        }
        let current = input.get();
        let rules = TaxRules::for_year(current.financial_year);
        let annual_target = target * target_freq.get().periods_per_year();
        solve_gross_for_net(annual_target, &current, &rules)
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

                <Collapsible title="Net to Gross" closed=true>
                    <p class="muted">
                        "Find the salary needed to take home a target net amount, using the settings above (year, residency, super, deductions)."
                    </p>
                    <label for="target-net">"Target net income (AUD)"</label>
                    <input
                        id="target-net"
                        type="number" inputmode="decimal"
                        min="0"
                        step="100"
                        prop:value=move || {
                            let v = target_net.get();
                            if v > 0.0 { v.to_string() } else { String::new() }
                        }
                        on:input=move |ev| {
                            target_net.set(event_target_value(&ev).parse::<f64>().unwrap_or(0.0));
                        }
                    />
                    <label for="target-net-freq">"Target is per"</label>
                    <select
                        id="target-net-freq"
                        on:change=move |ev| {
                            target_freq.set(match event_target_value(&ev).as_str() {
                                "weekly" => PayFrequency::Weekly,
                                "fortnightly" => PayFrequency::Fortnightly,
                                "monthly" => PayFrequency::Monthly,
                                _ => PayFrequency::Annually,
                            });
                        }
                    >
                        <option value="annually" selected=move || target_freq.get() == PayFrequency::Annually>"Year"</option>
                        <option value="monthly" selected=move || target_freq.get() == PayFrequency::Monthly>"Month"</option>
                        <option value="fortnightly" selected=move || target_freq.get() == PayFrequency::Fortnightly>"Fortnight"</option>
                        <option value="weekly" selected=move || target_freq.get() == PayFrequency::Weekly>"Week"</option>
                    </select>
                    {move || {
                        let target = target_net.get();
                        (target > 0.0).then(|| {
                            match required_gross.get() {
                                Some(gross) => {
                                    let label = if input.get().includes_super {
                                        "Required Package (Incl. Super)"
                                    } else {
                                        "Required Gross Salary"
                                    };
                                    view! {
                                        <div class="summary-grid">
                                            <article class="mini-card">
                                                <span class="muted">{label}</span>
                                                <strong>{format!("{} / year", fmt_money(gross))}</strong>
                                            </article>
                                        </div>
                                    }.into_view()
                                }
                                None => view! {
                                    <p class="error">"No achievable salary found for that target."</p>
                                }.into_view(),
                            }
                        })
                    }}
                </Collapsible>
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
                        let expenses_annual = load_budget_annual();
                        view! {
                            <TaxPieChart
                                result=result
                                mortgage_annual=mortgage_annual
                                debt_recycling_annual=debt_recycling_annual
                                expenses_annual=expenses_annual
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
