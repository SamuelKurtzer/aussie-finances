//! Shared cross-tab loaders: every page that needs another tab's data reads
//! it from localStorage through these helpers. Pages remount on tab switch,
//! so a mount-time read is always fresh.

use crate::domain::budget::BudgetInput;
use crate::domain::calculator::calculate_income;
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, first_year_repayments, load_income_context_from_saved_input,
    DebtRecycleInput, MortgagePortfolioInput, MortgagePortfolioOutput,
};
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorInput, CalculatorOutput};
use crate::storage::{
    load_from_storage, load_raw_from_storage, BUDGET_STORAGE_KEY, DEBT_RECYCLE_STORAGE_KEY,
    INCOME_STORAGE_KEY, MORTGAGE_STORAGE_KEY,
};

pub fn load_income_input() -> Option<CalculatorInput> {
    let raw = load_raw_from_storage(INCOME_STORAGE_KEY)?;
    serde_json::from_str::<CalculatorInput>(&raw).ok()
}

pub fn load_income_output() -> Option<CalculatorOutput> {
    let input = load_income_input()?;
    let rules = TaxRules::for_year(input.financial_year);
    calculate_income(&input, &rules).ok()
}

pub fn load_monthly_net_income() -> Option<f64> {
    load_income_output().map(|out| out.net_income_annual / 12.0)
}

/// Saved mortgage portfolio with the separately-stored debt recycle strategy
/// merged in.
pub fn load_mortgage_input() -> Option<MortgagePortfolioInput> {
    let raw = load_raw_from_storage(MORTGAGE_STORAGE_KEY)?;
    let mut portfolio = serde_json::from_str::<MortgagePortfolioInput>(&raw).ok()?;
    portfolio.debt_recycle = load_from_storage::<DebtRecycleInput>(DEBT_RECYCLE_STORAGE_KEY);
    Some(portfolio)
}

pub fn load_mortgage_output() -> Option<MortgagePortfolioOutput> {
    let portfolio = load_mortgage_input()?;
    let income_ctx = load_raw_from_storage(INCOME_STORAGE_KEY)
        .as_deref()
        .and_then(load_income_context_from_saved_input);
    calculate_mortgage_portfolio(&portfolio, income_ctx.as_ref()).ok()
}

pub fn load_budget_annual() -> f64 {
    load_from_storage::<BudgetInput>(BUDGET_STORAGE_KEY)
        .map(|b| b.annual_total())
        .unwrap_or(0.0)
}

/// (annual mortgage repayments, annual debt-recycle base outgoings).
pub fn load_household_outgoings() -> (f64, f64) {
    let Some(portfolio) = load_mortgage_input() else {
        return (0.0, 0.0);
    };

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
