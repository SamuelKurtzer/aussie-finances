//! Shared cross-tab loaders: every page that needs another tab's data reads
//! it from localStorage through these helpers. Pages remount on tab switch,
//! so a mount-time read is always fresh.

use crate::domain::budget::{monthly_surplus, BudgetInput};
use crate::domain::calculator::calculate_income;
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, first_year_dividend_cash, first_year_repayments,
    load_income_context_from_saved_input, monthly_amount_per_period, DebtRecycleInput,
    MortgagePortfolioInput, MortgagePortfolioOutput,
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

/// If the DR link is on, overwrite dividends/franking/company-rate from the
/// stored DR projection's first year. Recursion-safe by construction: the
/// projection here runs on the RAW mortgage input (no surplus top-up, no
/// income context), and dividends never feed back into the projection.
pub fn resolve_income_input(mut input: CalculatorInput) -> CalculatorInput {
    if !input.link_dividends_to_dr {
        return input;
    }
    let Some(portfolio) = load_mortgage_input() else {
        return input;
    };
    let Some(dr) = portfolio.debt_recycle.clone().filter(|d| d.enabled) else {
        return input;
    };
    if let Ok(out) = calculate_mortgage_portfolio(&portfolio, None) {
        if let Some(dr_out) = out.debt_recycle {
            input.dividends_annual =
                first_year_dividend_cash(&dr_out.periods, &out.chart_series.period_months);
            input.dividend_franking_percent = dr.franking_percent;
            input.dividend_company_tax_rate_percent = dr.company_tax_rate_percent;
        }
    }
    input
}

pub fn load_income_output() -> Option<CalculatorOutput> {
    let input = resolve_income_input(load_income_input()?);
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

/// Per-period offset top-up derived from the Budget tab's monthly surplus.
/// The repayment figure feeding the surplus is computed with the top-up
/// zeroed, which breaks the (circular) dependency of surplus on itself; P&I
/// repayments don't change with offset anyway, so this is exact until early
/// payoff.
pub fn derived_surplus_top_up_per_period(portfolio: &MortgagePortfolioInput) -> f64 {
    let Some(net_monthly) = load_monthly_net_income() else {
        return 0.0;
    };
    let budget = load_from_storage::<BudgetInput>(BUDGET_STORAGE_KEY).unwrap_or_default();

    let mut base = portfolio.clone();
    base.offset_top_up_per_period = 0.0;
    base.top_up_from_budget_surplus = false;
    let income_ctx = load_raw_from_storage(INCOME_STORAGE_KEY)
        .as_deref()
        .and_then(load_income_context_from_saved_input);
    let mortgage_monthly = calculate_mortgage_portfolio(&base, income_ctx.as_ref())
        .ok()
        .map(|out| {
            first_year_repayments(&out.amortization_rows, &out.chart_series.period_months) / 12.0
        })
        .unwrap_or(0.0);

    let dr_monthly = portfolio
        .debt_recycle
        .as_ref()
        .filter(|dr| dr.enabled)
        .map(|dr| dr.redraw_amount_aud / dr.redraw_cadence.interval_months())
        .unwrap_or(0.0);

    let surplus = monthly_surplus(&budget, net_monthly, mortgage_monthly, dr_monthly);
    monthly_amount_per_period(surplus, portfolio.repayment_cadence)
}

/// Saved portfolio with the surplus-derived offset top-up applied when the
/// user has opted in on the Loans tab.
pub fn effective_portfolio_input() -> Option<MortgagePortfolioInput> {
    let mut portfolio = load_mortgage_input()?;
    if portfolio.top_up_from_budget_surplus {
        portfolio.offset_top_up_per_period = derived_surplus_top_up_per_period(&portfolio);
    }
    Some(portfolio)
}

pub fn load_mortgage_output() -> Option<MortgagePortfolioOutput> {
    let portfolio = effective_portfolio_input()?;
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
