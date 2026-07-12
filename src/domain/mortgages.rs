use serde::{Deserialize, Serialize};

use crate::domain::calculator::calculate_income;
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorInput, DomainError, PayFrequency, ValidationIssue};

pub const MAX_MORTGAGES: usize = 10;
pub const MAX_SPLITS_PER_MORTGAGE: usize = 10;
const BALANCE_EPSILON: f64 = 1e-6;

pub type MortgageValidationError = DomainError;

#[inline]
fn balance_is_zero(value: f64) -> bool {
    value.abs() <= BALANCE_EPSILON
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RepaymentCadence {
    Weekly,
    Fortnightly,
    Monthly,
}

impl RepaymentCadence {
    pub fn periods_per_year(self) -> usize {
        match self {
            Self::Weekly => 52,
            Self::Fortnightly => 26,
            Self::Monthly => 12,
        }
    }
}

impl From<PayFrequency> for RepaymentCadence {
    fn from(value: PayFrequency) -> Self {
        match value {
            PayFrequency::Weekly => Self::Weekly,
            PayFrequency::Fortnightly => Self::Fortnightly,
            PayFrequency::Monthly | PayFrequency::Annually => Self::Monthly,
        }
    }
}

#[derive(Clone, Copy, Debug, Default, PartialEq, Eq, Serialize, Deserialize)]
pub enum RedrawCadence {
    #[default]
    Monthly,
    Quarterly,
    Yearly,
}

impl RedrawCadence {
    pub fn interval_months(self) -> f64 {
        match self {
            Self::Monthly => 1.0,
            Self::Quarterly => 3.0,
            Self::Yearly => 12.0,
        }
    }

    pub fn label(self) -> &'static str {
        match self {
            Self::Monthly => "Monthly",
            Self::Quarterly => "Quarterly",
            Self::Yearly => "Yearly",
        }
    }

    pub const ALL: [RedrawCadence; 3] = [Self::Monthly, Self::Quarterly, Self::Yearly];
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum RateType {
    Fixed,
    Variable,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanPurpose {
    OwnerOccupied,
    Investment,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum LoanRepaymentType {
    PrincipalAndInterest,
    InterestOnlyThenPrincipalAndInterest,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct SplitInput {
    pub id: u32,
    pub name: String,
    pub loan_amount: f64,
    pub annual_rate_percent: f64,
    pub rate_type: RateType,
    pub loan_purpose: LoanPurpose,
    pub repayment_type: LoanRepaymentType,
    pub interest_only_years: f64,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MortgageInput {
    pub id: u32,
    pub name: String,
    pub home_value: f64,
    pub offset_balance: f64,
    pub term_months: u32,
    pub splits: Vec<SplitInput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct MortgagePortfolioInput {
    pub repayment_cadence: RepaymentCadence,
    pub match_income_cadence: bool,
    pub offset_top_up_per_period: f64,
    #[serde(default)]
    pub debt_recycle: Option<DebtRecycleInput>,
    pub mortgages: Vec<MortgageInput>,
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct DebtRecycleInput {
    pub enabled: bool,
    pub mortgage_id: u32,
    /// Amount drawn per redraw event; the alias keeps pre-cadence saved
    /// data (monthly-only era) importable.
    #[serde(alias = "monthly_redraw_aud")]
    pub redraw_amount_aud: f64,
    pub redraw_cadence: RedrawCadence,
    pub emergency_buffer_aud: f64,
    pub growth_rate_percent: f64,
    pub dividend_yield_percent: f64,
    pub franking_percent: f64,
    pub company_tax_rate_percent: f64,
    pub starting_investment_aud: f64,
}

impl Default for DebtRecycleInput {
    fn default() -> Self {
        Self {
            enabled: false,
            mortgage_id: 1,
            redraw_amount_aud: 2_000.0,
            redraw_cadence: RedrawCadence::Monthly,
            emergency_buffer_aud: 20_000.0,
            growth_rate_percent: 6.0,
            dividend_yield_percent: 4.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebtRecyclePeriod {
    pub period_index: usize,
    pub redraw_amount: f64,
    pub investment_value: f64,
    pub dividend_cash: f64,
    pub franking_credit: f64,
    pub offset_before: f64,
    pub offset_after: f64,
    pub recycled_debt_balance: f64,
    pub deductible_interest: f64,
    pub cumulative_deductible_interest: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebtRecycleSummary {
    pub total_drawn: f64,
    pub draw_count: usize,
    pub ending_investment_value: f64,
    pub total_dividends: f64,
    pub total_franking_credits: f64,
    pub ending_recycled_debt_balance: f64,
    pub total_deductible_interest: f64,
    pub recycled_principal_repaid: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebtRecycleOutput {
    pub summary: DebtRecycleSummary,
    pub periods: Vec<DebtRecyclePeriod>,
    pub warnings: Vec<String>,
}

impl MortgagePortfolioInput {
    pub fn next_mortgage_id(&self) -> u32 {
        self.mortgages.iter().map(|m| m.id).max().unwrap_or(0) + 1
    }
}

impl MortgageInput {
    pub fn next_split_id(&self) -> u32 {
        self.splits.iter().map(|s| s.id).max().unwrap_or(0) + 1
    }
}

impl Default for SplitInput {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Split 1".to_string(),
            loan_amount: 500_000.0,
            annual_rate_percent: 6.0,
            rate_type: RateType::Variable,
            loan_purpose: LoanPurpose::OwnerOccupied,
            repayment_type: LoanRepaymentType::PrincipalAndInterest,
            interest_only_years: 0.0,
        }
    }
}

impl Default for MortgageInput {
    fn default() -> Self {
        Self {
            id: 1,
            name: "Mortgage 1".to_string(),
            home_value: 750_000.0,
            offset_balance: 20_000.0,
            term_months: 360,
            splits: vec![SplitInput::default()],
        }
    }
}

impl Default for MortgagePortfolioInput {
    fn default() -> Self {
        Self {
            repayment_cadence: RepaymentCadence::Fortnightly,
            match_income_cadence: true,
            offset_top_up_per_period: 0.0,
            debt_recycle: None,
            mortgages: vec![MortgageInput::default()],
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct IncomeContext {
    pub net_income_annual: f64,
    pub pay_frequency: PayFrequency,
}

#[derive(Clone, Debug, PartialEq)]
pub struct PortfolioTotals {
    pub total_debt: f64,
    pub total_property_value: f64,
    pub total_equity: f64,
    pub portfolio_lvr_percent: f64,
    pub periodic_repayment_total: f64,
    pub projected_total_interest: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MortgageSummary {
    pub mortgage_id: u32,
    pub mortgage_name: String,
    pub debt: f64,
    pub property_value: f64,
    pub equity: f64,
    pub lvr_percent: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct AmortizationRow {
    pub period_index: usize,
    pub opening_balance: f64,
    pub repayment: f64,
    pub interest: f64,
    pub principal: f64,
    pub closing_balance: f64,
    pub offset_balance: f64,
    pub cumulative_interest: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MortgageChartSeries {
    pub period_months: Vec<f64>,
    pub total_balance: Vec<f64>,
    pub worst_case_total_balance: Vec<f64>,
    pub cumulative_repayment: Vec<f64>,
    pub worst_case_cumulative_repayment: Vec<f64>,
    pub cumulative_interest: Vec<f64>,
    pub worst_case_cumulative_interest: Vec<f64>,
    pub offset_balance: Vec<f64>,
}

#[derive(Clone, Debug, PartialEq)]
pub struct MortgagePortfolioOutput {
    pub portfolio_totals: PortfolioTotals,
    pub mortgage_summaries: Vec<MortgageSummary>,
    pub amortization_rows: Vec<AmortizationRow>,
    pub worst_case_amortization_rows: Vec<AmortizationRow>,
    pub chart_series: MortgageChartSeries,
    pub debt_recycle: Option<DebtRecycleOutput>,
    pub repayment_to_income_percent: Option<f64>,
    pub warnings: Vec<String>,
}

#[derive(Clone)]
struct WorkingSplit {
    balance: f64,
    recycled_balance: f64,
    rate_per_period: f64,
    rate_type: RateType,
    io_periods: usize,
    repayment_type: LoanRepaymentType,
    loan_purpose: LoanPurpose,
    fixed_repayment: f64,
}

#[derive(Clone)]
struct WorkingMortgage {
    id: u32,
    term_periods: usize,
    offset_balance: f64,
    splits: Vec<WorkingSplit>,
}

#[derive(Clone, Copy)]
struct ProjectionDebtRecycleConfig {
    mortgage_index: usize,
    redraw_amount: f64,
    redraw_interval_months: f64,
    emergency_buffer: f64,
    growth_rate_per_period: f64,
    dividend_rate_per_period: f64,
    franking_multiplier: f64,
    starting_investment: f64,
}

struct ProjectionDebtRecycleState {
    config: ProjectionDebtRecycleConfig,
    periods: Vec<DebtRecyclePeriod>,
    total_drawn: f64,
    draw_count: usize,
    total_dividends: f64,
    total_franking_credits: f64,
    total_deductible_interest: f64,
    recycled_principal_repaid: f64,
    investment_value: f64,
    income_since_last_draw: f64,
    capacity_exhausted_warned: bool,
    warnings: Vec<String>,
}

struct ProjectionRun {
    rows: Vec<AmortizationRow>,
    period_months: Vec<f64>,
    balances: Vec<f64>,
    cumulative_repayment_series: Vec<f64>,
    cumulative_interest_series: Vec<f64>,
    offset_series: Vec<f64>,
    first_period_repayment: f64,
    cumulative_interest: f64,
    debt_recycle: Option<DebtRecycleOutput>,
}

pub fn load_income_context_from_saved_input(raw: &str) -> Option<IncomeContext> {
    let parsed = serde_json::from_str::<CalculatorInput>(raw).ok()?;
    let output = calculate_income(&parsed, &TaxRules::for_year(parsed.financial_year)).ok()?;
    Some(IncomeContext {
        net_income_annual: output.net_income_annual,
        pay_frequency: parsed.pay_frequency,
    })
}

pub fn calculate_mortgage_portfolio(
    input: &MortgagePortfolioInput,
    income_context: Option<&IncomeContext>,
) -> Result<MortgagePortfolioOutput, MortgageValidationError> {
    let issues = validate_portfolio_input(input);
    if !issues.is_empty() {
        return Err(MortgageValidationError::Validation(issues));
    }

    let periods_per_year = input.repayment_cadence.periods_per_year();
    let mut warnings = Vec::new();
    let mut mortgage_summaries = Vec::new();

    let mut total_debt = 0.0;
    let mut total_property_value = 0.0;
    let mut total_equity = 0.0;

    let mut working = Vec::new();

    for mortgage in &input.mortgages {
        let debt = mortgage.splits.iter().map(|s| s.loan_amount).sum::<f64>();
        let property_value = mortgage.home_value;
        let equity = property_value - debt;
        let lvr = if property_value > 0.0 {
            (debt / property_value) * 100.0
        } else {
            0.0
        };

        if lvr > 80.0 {
            warnings.push(format!(
                "{} has LVR {:.1}% which is above 80%.",
                mortgage.name, lvr
            ));
        }

        total_debt += debt;
        total_property_value += property_value;
        total_equity += equity;

        mortgage_summaries.push(MortgageSummary {
            mortgage_id: mortgage.id,
            mortgage_name: mortgage.name.clone(),
            debt,
            property_value,
            equity,
            lvr_percent: lvr,
        });

        let term_periods = term_months_to_periods(mortgage.term_months, input.repayment_cadence);
        let splits = mortgage
            .splits
            .iter()
            .map(|split| {
                let rate_per_period = split.annual_rate_percent / 100.0 / periods_per_year as f64;
                let io_periods =
                    (split.interest_only_years * periods_per_year as f64).round() as usize;
                let fixed_repayment = match split.repayment_type {
                    LoanRepaymentType::PrincipalAndInterest => {
                        amortized_repayment(split.loan_amount, rate_per_period, term_periods)
                    }
                    LoanRepaymentType::InterestOnlyThenPrincipalAndInterest => 0.0,
                };
                WorkingSplit {
                    balance: split.loan_amount,
                    recycled_balance: 0.0,
                    rate_per_period,
                    rate_type: split.rate_type,
                    io_periods,
                    repayment_type: split.repayment_type,
                    loan_purpose: split.loan_purpose,
                    fixed_repayment,
                }
            })
            .collect::<Vec<_>>();

        working.push(WorkingMortgage {
            id: mortgage.id,
            term_periods,
            offset_balance: mortgage.offset_balance,
            splits,
        });
    }

    let debt_recycle_config = build_projection_debt_recycle_config(
        input,
        &working,
        input.repayment_cadence,
        &mut warnings,
    );

    let max_periods = working.iter().map(|w| w.term_periods).max().unwrap_or(0);
    let baseline_projection = run_projection(
        working.clone(),
        max_periods,
        input.offset_top_up_per_period,
        input.repayment_cadence,
        debt_recycle_config,
    );
    let mut worst_working = working;
    for wm in &mut worst_working {
        wm.offset_balance = 0.0;
    }
    let worst_projection = run_projection(
        worst_working,
        max_periods,
        0.0,
        input.repayment_cadence,
        None,
    );

    let portfolio_lvr_percent = if total_property_value > 0.0 {
        (total_debt / total_property_value) * 100.0
    } else {
        0.0
    };

    let repayment_to_income_percent = income_context
        .and_then(|ctx| {
            if input.match_income_cadence {
                map_income_to_cadence(ctx, RepaymentCadence::from(ctx.pay_frequency))
            } else {
                map_income_to_cadence(ctx, input.repayment_cadence)
            }
        })
        .and_then(|income_per_period| {
            if income_per_period > 0.0 {
                Some((baseline_projection.first_period_repayment / income_per_period) * 100.0)
            } else {
                None
            }
        });

    if income_context.is_none() {
        warnings.push("Income data not found. Complete the Income Calculator first to see repayment-to-income %.".to_string());
    }

    let debt_recycle_output = baseline_projection.debt_recycle;
    if let Some(debt_recycle) = debt_recycle_output.as_ref() {
        warnings.extend(debt_recycle.warnings.iter().cloned());
    }

    Ok(MortgagePortfolioOutput {
        portfolio_totals: PortfolioTotals {
            total_debt,
            total_property_value,
            total_equity,
            portfolio_lvr_percent,
            periodic_repayment_total: baseline_projection.first_period_repayment,
            projected_total_interest: baseline_projection.cumulative_interest,
        },
        mortgage_summaries,
        amortization_rows: baseline_projection.rows,
        worst_case_amortization_rows: worst_projection.rows,
        chart_series: MortgageChartSeries {
            period_months: baseline_projection.period_months,
            total_balance: baseline_projection.balances,
            worst_case_total_balance: worst_projection.balances,
            cumulative_repayment: baseline_projection.cumulative_repayment_series,
            worst_case_cumulative_repayment: worst_projection.cumulative_repayment_series,
            cumulative_interest: baseline_projection.cumulative_interest_series,
            worst_case_cumulative_interest: worst_projection.cumulative_interest_series,
            offset_balance: baseline_projection.offset_series,
        },
        debt_recycle: debt_recycle_output,
        repayment_to_income_percent,
        warnings,
    })
}

fn build_projection_debt_recycle_config(
    input: &MortgagePortfolioInput,
    working: &[WorkingMortgage],
    cadence: RepaymentCadence,
    warnings: &mut Vec<String>,
) -> Option<ProjectionDebtRecycleConfig> {
    let Some(debt_recycle) = input.debt_recycle.as_ref() else {
        return None;
    };
    if !debt_recycle.enabled {
        return None;
    }

    let Some(mortgage_index) = working
        .iter()
        .position(|m| m.id == debt_recycle.mortgage_id)
    else {
        warnings
            .push("Debt recycle strategy skipped: selected mortgage was not found.".to_string());
        return None;
    };

    let Some(mortgage_input) = input
        .mortgages
        .iter()
        .find(|m| m.id == debt_recycle.mortgage_id)
    else {
        warnings.push(
            "Debt recycle strategy skipped: selected mortgage input was not found.".to_string(),
        );
        return None;
    };

    let has_eligible_split = mortgage_input.splits.iter().any(|s| {
        s.rate_type == RateType::Variable && s.loan_purpose == LoanPurpose::OwnerOccupied
    });
    if !has_eligible_split {
        warnings.push(format!(
            "Debt recycle strategy skipped: {} has no variable owner-occupied split to pay into and redraw from.",
            mortgage_input.name
        ));
        return None;
    }

    let periods_per_year = cadence.periods_per_year() as f64;
    let growth_rate_per_period =
        (1.0 + debt_recycle.growth_rate_percent / 100.0).powf(1.0 / periods_per_year) - 1.0;
    let dividend_rate_per_period =
        (1.0 + debt_recycle.dividend_yield_percent / 100.0).powf(1.0 / periods_per_year) - 1.0;
    let franking_ratio = debt_recycle.franking_percent / 100.0;
    let company_tax_ratio = debt_recycle.company_tax_rate_percent / 100.0;
    let franking_multiplier = franking_ratio * company_tax_ratio / (1.0 - company_tax_ratio);

    Some(ProjectionDebtRecycleConfig {
        mortgage_index,
        redraw_amount: debt_recycle.redraw_amount_aud,
        redraw_interval_months: debt_recycle.redraw_cadence.interval_months(),
        emergency_buffer: debt_recycle.emergency_buffer_aud,
        growth_rate_per_period,
        dividend_rate_per_period,
        franking_multiplier,
        starting_investment: debt_recycle.starting_investment_aud.max(0.0),
    })
}

fn run_projection(
    mut working: Vec<WorkingMortgage>,
    max_periods: usize,
    offset_top_up_per_period: f64,
    cadence: RepaymentCadence,
    debt_recycle_config: Option<ProjectionDebtRecycleConfig>,
) -> ProjectionRun {
    let mut rows = Vec::with_capacity(max_periods);
    let mut period_months = Vec::with_capacity(max_periods + 1);
    let mut balances = Vec::with_capacity(max_periods + 1);
    let mut cumulative_repayment_series = Vec::with_capacity(max_periods + 1);
    let mut cumulative_interest_series = Vec::with_capacity(max_periods + 1);
    let mut offset_series = Vec::with_capacity(max_periods + 1);

    let mut cumulative_repayment = 0.0;
    let mut cumulative_interest = 0.0;
    let mut first_period_repayment = 0.0;
    let mut debt_recycle_state = debt_recycle_config.map(|config| ProjectionDebtRecycleState {
        config,
        periods: Vec::with_capacity(max_periods),
        total_drawn: 0.0,
        draw_count: 0,
        total_dividends: 0.0,
        total_franking_credits: 0.0,
        total_deductible_interest: 0.0,
        recycled_principal_repaid: 0.0,
        investment_value: config.starting_investment,
        income_since_last_draw: 0.0,
        capacity_exhausted_warned: false,
        warnings: Vec::new(),
    });
    let months_per_period = 12.0 / cadence.periods_per_year() as f64;

    period_months.push(0.0);
    balances.push(total_remaining_balance(&working));
    cumulative_repayment_series.push(0.0);
    cumulative_interest_series.push(0.0);
    offset_series.push(working.iter().map(|m| m.offset_balance).sum());

    for period in 1..=max_periods {
        for wm in &mut working {
            wm.offset_balance += offset_top_up_per_period;
        }

        let mut recycle_snapshot = None;
        if let Some(state) = &mut debt_recycle_state {
            let target_mortgage = &mut working[state.config.mortgage_index];
            let dividend_cash =
                (state.investment_value * state.config.dividend_rate_per_period).max(0.0);
            let franking_credit = (dividend_cash * state.config.franking_multiplier).max(0.0);
            target_mortgage.offset_balance += dividend_cash + franking_credit;
            state.total_dividends += dividend_cash;
            state.total_franking_credits += franking_credit;
            state.income_since_last_draw += dividend_cash + franking_credit;

            let offset_before = target_mortgage.offset_balance;
            let mut redraw_amount = 0.0;

            // A redraw fires each time a whole cadence interval (month,
            // quarter, or year) of elapsed time has completed.
            let interval = state.config.redraw_interval_months;
            let is_new_interval = (period as f64 * months_per_period / interval).floor()
                > ((period - 1) as f64 * months_per_period / interval).floor();
            if is_new_interval {
                let available_offset =
                    (target_mortgage.offset_balance - state.config.emergency_buffer).max(0.0);
                let capacity = redraw_capacity(target_mortgage);
                // Recycle strategy income (dividends + franking) on top of
                // the configured base amount, so the offset converges to the
                // emergency buffer instead of accumulating investment income;
                // the redraw grows over time as that income compounds.
                let requested = (state.config.redraw_amount + state.income_since_last_draw)
                    .min(available_offset)
                    .min(capacity);
                if requested > 0.0 && !balance_is_zero(requested) {
                    redraw_amount = apply_non_split_redraw(target_mortgage, requested);
                    target_mortgage.offset_balance -= redraw_amount;
                    state.investment_value += redraw_amount;
                    state.total_drawn += redraw_amount;
                    state.draw_count += 1;
                }
                state.income_since_last_draw = 0.0;
                if balance_is_zero(capacity) && !state.capacity_exhausted_warned {
                    state.capacity_exhausted_warned = true;
                    state.warnings.push(format!(
                        "Owner-occupied variable debt fully recycled by period {period}; no further redraws possible."
                    ));
                }
            }

            let offset_after = target_mortgage.offset_balance;
            recycle_snapshot = Some((
                redraw_amount,
                dividend_cash,
                franking_credit,
                offset_before,
                offset_after,
            ));
        }

        let opening_balance = total_remaining_balance(&working);
        let mut period_interest = 0.0;
        let mut period_principal = 0.0;
        let mut period_repayment = 0.0;
        let mut period_deductible_interest = 0.0;
        let mut period_recycled_principal = 0.0;

        for wm in &mut working {
            if period > wm.term_periods {
                continue;
            }

            let total_split_balance = wm.splits.iter().map(|s| s.balance).sum::<f64>();
            if balance_is_zero(total_split_balance) {
                continue;
            }

            let effective_offset = wm.offset_balance.min(total_split_balance);

            for split in &mut wm.splits {
                if balance_is_zero(split.balance) {
                    continue;
                }

                let balance_share = if total_split_balance > 0.0 {
                    split.balance / total_split_balance
                } else {
                    0.0
                };

                let split_offset = effective_offset * balance_share;
                let interest_base = (split.balance - split_offset).max(0.0);
                let interest = interest_base * split.rate_per_period;

                let repayment = match split.repayment_type {
                    LoanRepaymentType::PrincipalAndInterest => {
                        split.fixed_repayment.min(interest + split.balance)
                    }
                    LoanRepaymentType::InterestOnlyThenPrincipalAndInterest => {
                        if period <= split.io_periods {
                            interest
                        } else {
                            let remaining_periods = wm.term_periods - period + 1;
                            if split.fixed_repayment <= 0.0 {
                                split.fixed_repayment = amortized_repayment(
                                    split.balance,
                                    split.rate_per_period,
                                    remaining_periods,
                                );
                            }
                            split.fixed_repayment.min(interest + split.balance)
                        }
                    }
                };

                let principal = (repayment - interest).max(0.0).min(split.balance);

                // ATO mixed-purpose loan treatment: interest and principal
                // repayments apportion pro-rata across the deductible
                // (recycled) and non-deductible portions of the loan.
                if split.recycled_balance > 0.0 && split.balance > 0.0 {
                    let recycled_share = (split.recycled_balance / split.balance).min(1.0);
                    period_deductible_interest += interest * recycled_share;
                    let recycled_principal = principal * recycled_share;
                    split.recycled_balance =
                        (split.recycled_balance - recycled_principal).max(0.0);
                    period_recycled_principal += recycled_principal;
                }

                split.balance -= principal;
                split.recycled_balance = split.recycled_balance.min(split.balance);

                period_interest += interest;
                period_principal += principal;
                period_repayment += interest + principal;
            }
        }

        if let Some(state) = &mut debt_recycle_state {
            state.investment_value *= 1.0 + state.config.growth_rate_per_period;
            state.total_deductible_interest += period_deductible_interest;
            state.recycled_principal_repaid += period_recycled_principal;
            if let Some((redraw_amount, dividend_cash, franking_credit, offset_before, offset_after)) =
                recycle_snapshot
            {
                state.periods.push(DebtRecyclePeriod {
                    period_index: period,
                    redraw_amount,
                    investment_value: state.investment_value,
                    dividend_cash,
                    franking_credit,
                    offset_before,
                    offset_after,
                    recycled_debt_balance: recycled_debt_balance(&working),
                    deductible_interest: period_deductible_interest,
                    cumulative_deductible_interest: state.total_deductible_interest,
                });
            }
        }

        let closing_balance = total_remaining_balance(&working);
        cumulative_repayment += period_repayment;
        cumulative_interest += period_interest;

        if period == 1 {
            first_period_repayment = period_repayment;
        }

        rows.push(AmortizationRow {
            period_index: period,
            opening_balance,
            repayment: period_repayment,
            interest: period_interest,
            principal: period_principal,
            closing_balance,
            offset_balance: working.iter().map(|m| m.offset_balance).sum(),
            cumulative_interest,
        });

        let months_per_period = 12.0 / cadence.periods_per_year() as f64;
        period_months.push(period as f64 * months_per_period);
        balances.push(closing_balance);
        cumulative_repayment_series.push(cumulative_repayment);
        cumulative_interest_series.push(cumulative_interest);
        offset_series.push(working.iter().map(|m| m.offset_balance).sum());
    }

    ProjectionRun {
        rows,
        period_months,
        balances,
        cumulative_repayment_series,
        cumulative_interest_series,
        offset_series,
        first_period_repayment,
        cumulative_interest,
        debt_recycle: debt_recycle_state.map(|state| DebtRecycleOutput {
            summary: DebtRecycleSummary {
                total_drawn: state.total_drawn,
                draw_count: state.draw_count,
                ending_investment_value: state.investment_value,
                total_dividends: state.total_dividends,
                total_franking_credits: state.total_franking_credits,
                ending_recycled_debt_balance: recycled_debt_balance(&working),
                total_deductible_interest: state.total_deductible_interest,
                recycled_principal_repaid: state.recycled_principal_repaid,
            },
            periods: state.periods,
            warnings: state.warnings,
        }),
    }
}

fn recycled_debt_balance(working: &[WorkingMortgage]) -> f64 {
    working
        .iter()
        .flat_map(|m| m.splits.iter())
        .map(|s| s.recycled_balance)
        .sum()
}

fn split_is_redraw_eligible(split: &WorkingSplit) -> bool {
    split.rate_type == RateType::Variable && split.loan_purpose == LoanPurpose::OwnerOccupied
}

fn redraw_capacity(mortgage: &WorkingMortgage) -> f64 {
    mortgage
        .splits
        .iter()
        .filter(|s| split_is_redraw_eligible(s))
        .map(|s| (s.balance - s.recycled_balance).max(0.0))
        .sum()
}

// Pay-in then redraw nets to zero on the loan balance; the redrawn portion
// becomes deductible, tracked as recycled_balance within the same split.
fn apply_non_split_redraw(mortgage: &mut WorkingMortgage, requested: f64) -> f64 {
    let mut remaining = requested.max(0.0);
    if balance_is_zero(remaining) {
        return 0.0;
    }

    for split in mortgage
        .splits
        .iter_mut()
        .filter(|s| split_is_redraw_eligible(s))
    {
        let capacity = (split.balance - split.recycled_balance).max(0.0);
        let taken = capacity.min(remaining);
        split.recycled_balance += taken;
        remaining -= taken;
        if balance_is_zero(remaining) {
            break;
        }
    }

    requested.max(0.0) - remaining.max(0.0)
}

fn total_remaining_balance(working: &[WorkingMortgage]) -> f64 {
    working
        .iter()
        .flat_map(|m| m.splits.iter())
        .map(|s| s.balance)
        .sum()
}

fn amortized_repayment(balance: f64, rate_per_period: f64, remaining_periods: usize) -> f64 {
    if remaining_periods == 0 {
        return balance;
    }
    if rate_per_period == 0.0 {
        return balance / remaining_periods as f64;
    }
    let factor = (1.0 + rate_per_period).powf(-(remaining_periods as f64));
    balance * rate_per_period / (1.0 - factor)
}

fn map_income_to_cadence(ctx: &IncomeContext, cadence: RepaymentCadence) -> Option<f64> {
    let periods = cadence.periods_per_year() as f64;
    if periods <= 0.0 {
        return None;
    }
    Some(ctx.net_income_annual / periods)
}

fn term_months_to_periods(term_months: u32, cadence: RepaymentCadence) -> usize {
    let months = term_months as f64;
    let periods = match cadence {
        RepaymentCadence::Monthly => months,
        RepaymentCadence::Fortnightly => months * (26.0 / 12.0),
        RepaymentCadence::Weekly => months * (52.0 / 12.0),
    };
    periods.round().max(1.0) as usize
}

pub fn validate_portfolio_input(input: &MortgagePortfolioInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if input.mortgages.is_empty() {
        issues.push(ValidationIssue {
            field: "mortgages",
            message: "Add at least one mortgage.".to_string(),
        });
    }

    if input.mortgages.len() > MAX_MORTGAGES {
        issues.push(ValidationIssue {
            field: "mortgages",
            message: format!("Maximum {} mortgages allowed.", MAX_MORTGAGES),
        });
    }

    if input.offset_top_up_per_period < 0.0 {
        issues.push(ValidationIssue {
            field: "offset_top_up_per_period",
            message: "Offset top-up per period must be zero or greater.".to_string(),
        });
    }

    if let Some(debt_recycle) = input.debt_recycle.as_ref().filter(|d| d.enabled) {
        if debt_recycle.redraw_amount_aud <= 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.redraw_amount_aud",
                message: "Debt recycle redraw amount must be greater than zero.".to_string(),
            });
        }
        if debt_recycle.emergency_buffer_aud < 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.emergency_buffer_aud",
                message: "Debt recycle emergency buffer must be zero or greater.".to_string(),
            });
        }
        if debt_recycle.growth_rate_percent < 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.growth_rate_percent",
                message: "Debt recycle growth rate must be zero or greater.".to_string(),
            });
        }
        if debt_recycle.dividend_yield_percent < 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.dividend_yield_percent",
                message: "Debt recycle dividend yield must be zero or greater.".to_string(),
            });
        }
        if debt_recycle.franking_percent < 0.0 || debt_recycle.franking_percent > 100.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.franking_percent",
                message: "Debt recycle franking percent must be between 0 and 100.".to_string(),
            });
        }
        if debt_recycle.company_tax_rate_percent <= 0.0
            || debt_recycle.company_tax_rate_percent >= 100.0
        {
            issues.push(ValidationIssue {
                field: "debt_recycle.company_tax_rate_percent",
                message: "Debt recycle company tax rate must be greater than 0 and less than 100."
                    .to_string(),
            });
        }
        if debt_recycle.starting_investment_aud < 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.starting_investment_aud",
                message: "Debt recycle starting investment must be zero or greater.".to_string(),
            });
        }
        if !input
            .mortgages
            .iter()
            .any(|m| m.id == debt_recycle.mortgage_id)
        {
            issues.push(ValidationIssue {
                field: "debt_recycle.mortgage_id",
                message: "Debt recycle selected mortgage was not found.".to_string(),
            });
        }
    }

    for (mi, mortgage) in input.mortgages.iter().enumerate() {
        if mortgage.home_value <= 0.0 {
            issues.push(ValidationIssue {
                field: "home_value",
                message: format!("Mortgage {} home value must be greater than zero.", mi + 1),
            });
        }
        if mortgage.offset_balance < 0.0 {
            issues.push(ValidationIssue {
                field: "offset_balance",
                message: format!("Mortgage {} offset must be zero or greater.", mi + 1),
            });
        }
        if mortgage.term_months == 0 {
            issues.push(ValidationIssue {
                field: "term_months",
                message: format!("Mortgage {} term months must be greater than zero.", mi + 1),
            });
        }
        if mortgage.splits.is_empty() {
            issues.push(ValidationIssue {
                field: "splits",
                message: format!("Mortgage {} needs at least one split.", mi + 1),
            });
        }
        if mortgage.splits.len() > MAX_SPLITS_PER_MORTGAGE {
            issues.push(ValidationIssue {
                field: "splits",
                message: format!(
                    "Mortgage {} exceeds max {} splits.",
                    mi + 1,
                    MAX_SPLITS_PER_MORTGAGE
                ),
            });
        }

        for (si, split) in mortgage.splits.iter().enumerate() {
            if split.loan_amount <= 0.0 {
                issues.push(ValidationIssue {
                    field: "loan_amount",
                    message: format!(
                        "Mortgage {} split {} loan amount must be greater than zero.",
                        mi + 1,
                        si + 1
                    ),
                });
            }
            if split.annual_rate_percent < 0.0 {
                issues.push(ValidationIssue {
                    field: "annual_rate_percent",
                    message: format!(
                        "Mortgage {} split {} rate must be zero or greater.",
                        mi + 1,
                        si + 1
                    ),
                });
            }
            if split.interest_only_years < 0.0 {
                issues.push(ValidationIssue {
                    field: "interest_only_years",
                    message: format!(
                        "Mortgage {} split {} IO years must be zero or greater.",
                        mi + 1,
                        si + 1
                    ),
                });
            }
            if split.repayment_type == LoanRepaymentType::InterestOnlyThenPrincipalAndInterest
                && split.interest_only_years >= (mortgage.term_months as f64 / 12.0)
            {
                issues.push(ValidationIssue {
                    field: "interest_only_years",
                    message: format!(
                        "Mortgage {} split {} IO years must be less than mortgage term.",
                        mi + 1,
                        si + 1
                    ),
                });
            }
        }

        let debt = mortgage.splits.iter().map(|s| s.loan_amount).sum::<f64>();
        if debt > mortgage.home_value {
            issues.push(ValidationIssue {
                field: "home_value",
                message: format!("Mortgage {} total loan amount exceeds home value.", mi + 1),
            });
        }
    }

    issues
}

pub fn build_monthly_payment_series(
    rows: &[AmortizationRow],
    period_months: &[f64],
    initial_offset_balance: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    if rows.is_empty() {
        return (vec![0.0], vec![0.0], vec![0.0], vec![0.0]);
    }

    let month_series: Vec<f64> = if period_months.len() == rows.len() + 1 {
        period_months.to_vec()
    } else {
        (0..=rows.len()).map(|i| i as f64).collect()
    };

    let max_month = month_series
        .last()
        .copied()
        .unwrap_or(rows.len() as f64)
        .ceil() as usize;
    let mut principal = vec![0.0_f64; max_month + 1];
    let mut interest = vec![0.0_f64; max_month + 1];
    let mut offset_top_up = vec![0.0_f64; max_month + 1];

    let mut previous_offset = initial_offset_balance.max(0.0);
    for (i, row) in rows.iter().enumerate() {
        let delta_offset = (row.offset_balance - previous_offset).max(0.0);
        previous_offset = row.offset_balance;

        let start = month_series.get(i).copied().unwrap_or(i as f64);
        let end = month_series.get(i + 1).copied().unwrap_or((i + 1) as f64);
        let duration = (end - start).max(1e-9);
        let principal_rate = row.principal / duration;
        let interest_rate = row.interest / duration;
        let offset_rate = delta_offset / duration;

        let first_month = start.floor().max(0.0) as usize + 1;
        let last_month = end.ceil().max(0.0) as usize;

        for month in first_month..=last_month.min(max_month) {
            let left = (month - 1) as f64;
            let right = month as f64;
            let overlap = (end.min(right) - start.max(left)).max(0.0);
            if overlap <= 0.0 {
                continue;
            }
            principal[month] += principal_rate * overlap;
            interest[month] += interest_rate * overlap;
            offset_top_up[month] += offset_rate * overlap;
        }
    }

    let months = (0..=max_month).map(|m| m as f64).collect::<Vec<_>>();
    (months, principal, interest, offset_top_up)
}

pub fn first_year_repayments(rows: &[AmortizationRow], period_months: &[f64]) -> f64 {
    let aligned = period_months.len() == rows.len() + 1;
    rows.iter()
        .enumerate()
        .map(|(i, row)| {
            let (start, end) = if aligned {
                (period_months[i], period_months[i + 1])
            } else {
                (i as f64, (i + 1) as f64)
            };
            if start >= 12.0 {
                return 0.0;
            }
            let duration = (end - start).max(1e-9);
            let overlap = (end.min(12.0) - start).max(0.0);
            row.repayment * (overlap / duration)
        })
        .sum()
}

impl DebtRecycleInput {
    pub fn normalize_mortgage_selection(&mut self, portfolio: &MortgagePortfolioInput) {
        if portfolio
            .mortgages
            .iter()
            .any(|m| m.id == self.mortgage_id)
        {
            return;
        }
        self.mortgage_id = portfolio.mortgages.first().map(|m| m.id).unwrap_or(0);
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::domain::types::PayFrequency;

    fn base_input() -> MortgagePortfolioInput {
        MortgagePortfolioInput::default()
    }

    #[test]
    fn projection_reduces_balance_over_time() {
        let input = base_input();
        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let first = out
            .chart_series
            .total_balance
            .first()
            .copied()
            .unwrap_or_default();
        let last = out
            .chart_series
            .total_balance
            .last()
            .copied()
            .unwrap_or_default();
        assert!(last < first);
    }

    #[test]
    fn first_year_repayments_annualises_first_twelve_months() {
        let input = base_input();
        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let annual =
            first_year_repayments(&out.amortization_rows, &out.chart_series.period_months);
        let per_period = out.amortization_rows[0].repayment;
        let periods_per_year = input.repayment_cadence.periods_per_year() as f64;
        assert_relative_eq!(annual, per_period * periods_per_year, max_relative = 0.01);
    }

    #[test]
    fn offset_top_up_reduces_total_interest() {
        let mut a = base_input();
        a.offset_top_up_per_period = 0.0;
        let mut b = base_input();
        b.offset_top_up_per_period = 500.0;

        let out_a = calculate_mortgage_portfolio(&a, None).unwrap();
        let out_b = calculate_mortgage_portfolio(&b, None).unwrap();
        assert!(
            out_b.portfolio_totals.projected_total_interest
                < out_a.portfolio_totals.projected_total_interest
        );
    }

    #[test]
    fn repayment_to_income_works() {
        let input = base_input();
        let income = IncomeContext {
            net_income_annual: 120_000.0,
            pay_frequency: PayFrequency::Fortnightly,
        };
        let out = calculate_mortgage_portfolio(&input, Some(&income)).unwrap();
        assert!(out.repayment_to_income_percent.unwrap_or_default() > 0.0);
    }

    #[test]
    fn income_cadence_toggle_changes_ratio_basis() {
        let mut input = base_input();
        input.repayment_cadence = RepaymentCadence::Monthly;
        input.match_income_cadence = true;
        let income = IncomeContext {
            net_income_annual: 120_000.0,
            pay_frequency: PayFrequency::Weekly,
        };

        let matched = calculate_mortgage_portfolio(&input, Some(&income)).unwrap();
        input.match_income_cadence = false;
        let unmached = calculate_mortgage_portfolio(&input, Some(&income)).unwrap();
        assert!(
            (matched.repayment_to_income_percent.unwrap_or_default()
                - unmached.repayment_to_income_percent.unwrap_or_default())
            .abs()
                > 0.01
        );
    }

    #[test]
    fn cadence_changes_payment() {
        let mut monthly = base_input();
        monthly.repayment_cadence = RepaymentCadence::Monthly;
        let mut weekly = base_input();
        weekly.repayment_cadence = RepaymentCadence::Weekly;

        let mo = calculate_mortgage_portfolio(&monthly, None).unwrap();
        let we = calculate_mortgage_portfolio(&weekly, None).unwrap();
        assert!(
            mo.portfolio_totals.periodic_repayment_total
                > we.portfolio_totals.periodic_repayment_total
        );
    }

    #[test]
    fn io_then_pi_behaves() {
        let mut input = base_input();
        input.mortgages[0].splits[0].repayment_type =
            LoanRepaymentType::InterestOnlyThenPrincipalAndInterest;
        input.mortgages[0].splits[0].interest_only_years = 1.0;

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let first = &out.amortization_rows[0];
        assert_relative_eq!(first.principal, 0.0, epsilon = 0.01);
    }

    fn recycle_input(mortgage_id: u32) -> DebtRecycleInput {
        DebtRecycleInput {
            enabled: true,
            mortgage_id,
            redraw_amount_aud: 2_000.0,
            redraw_cadence: RedrawCadence::Monthly,
            emergency_buffer_aud: 5_000.0,
            growth_rate_percent: 0.0,
            dividend_yield_percent: 0.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 0.0,
        }
    }

    #[test]
    fn monthly_redraw_draws_until_offset_hits_buffer() {
        let mut input = base_input();
        input.debt_recycle = Some(recycle_input(input.mortgages[0].id));

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        // Offset 20k, buffer 5k: 7 full draws of 2k then one capped 1k draw.
        assert_eq!(recycle.summary.draw_count, 8);
        assert_relative_eq!(recycle.summary.total_drawn, 15_000.0, epsilon = 0.01);
        assert!(recycle.summary.ending_investment_value >= 15_000.0 - 0.01);
    }

    #[test]
    fn offset_stays_pinned_to_buffer_once_reached() {
        let mut input = base_input();
        input.repayment_cadence = RepaymentCadence::Monthly;
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.dividend_yield_percent = 6.0;
        recycle.starting_investment_aud = 100_000.0;
        input.debt_recycle = Some(recycle);

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");

        // Investment income is recycled on top of the base redraw, so after
        // the initial surplus is drawn down the offset sits at the buffer
        // right after every redraw instead of accumulating dividends.
        let draw_events: Vec<_> = recycle
            .periods
            .iter()
            .filter(|p| p.redraw_amount > 0.0)
            .collect();
        assert!(draw_events.len() > 24);
        for event in &draw_events[12..24] {
            assert_relative_eq!(event.offset_after, 5_000.0, epsilon = 0.01);
        }
        // And the redraw itself grows as the investment income compounds.
        assert!(draw_events[23].redraw_amount > draw_events[12].redraw_amount);
    }

    #[test]
    fn monthly_redraw_is_debt_neutral() {
        let mut input = base_input();
        input.mortgages[0].splits[0].repayment_type =
            LoanRepaymentType::InterestOnlyThenPrincipalAndInterest;
        input.mortgages[0].splits[0].interest_only_years = 29.0;
        input.debt_recycle = Some(recycle_input(input.mortgages[0].id));

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let first = out
            .chart_series
            .total_balance
            .first()
            .copied()
            .unwrap_or_default();
        let after_first = out
            .chart_series
            .total_balance
            .get(1)
            .copied()
            .unwrap_or_default();
        assert_relative_eq!(first, after_first, epsilon = 0.01);
    }

    #[test]
    fn monthly_redraw_events_follow_month_boundaries() {
        let mut input = base_input();
        input.repayment_cadence = RepaymentCadence::Fortnightly;
        input.mortgages[0].term_months = 12;
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.redraw_amount_aud = 1_000.0;
        recycle.emergency_buffer_aud = 0.0;
        input.debt_recycle = Some(recycle);

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        // 26 fortnights cover 12 month boundaries.
        assert_eq!(recycle.summary.draw_count, 12);
    }

    #[test]
    fn quarterly_redraw_fires_once_per_quarter() {
        let mut input = base_input();
        input.repayment_cadence = RepaymentCadence::Monthly;
        input.mortgages[0].term_months = 12;
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.redraw_amount_aud = 1_000.0;
        recycle.redraw_cadence = RedrawCadence::Quarterly;
        recycle.emergency_buffer_aud = 0.0;
        input.debt_recycle = Some(recycle);

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        assert_eq!(recycle.summary.draw_count, 4);
        // Draws land at the end of each quarter (months 3, 6, 9, 12).
        let draw_periods: Vec<usize> = recycle
            .periods
            .iter()
            .filter(|p| p.redraw_amount > 0.0)
            .map(|p| p.period_index)
            .collect();
        assert_eq!(draw_periods, vec![3, 6, 9, 12]);
    }

    #[test]
    fn quarterly_redraw_follows_quarter_boundaries_on_fortnightly_cadence() {
        let mut input = base_input();
        input.repayment_cadence = RepaymentCadence::Fortnightly;
        input.mortgages[0].term_months = 12;
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.redraw_amount_aud = 1_000.0;
        recycle.redraw_cadence = RedrawCadence::Quarterly;
        recycle.emergency_buffer_aud = 0.0;
        input.debt_recycle = Some(recycle);

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        // 26 fortnights cover 4 quarter boundaries.
        assert_eq!(recycle.summary.draw_count, 4);
    }

    #[test]
    fn yearly_redraw_fires_once_per_year() {
        let mut input = base_input();
        input.repayment_cadence = RepaymentCadence::Monthly;
        input.mortgages[0].term_months = 36;
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.redraw_amount_aud = 1_000.0;
        recycle.redraw_cadence = RedrawCadence::Yearly;
        recycle.emergency_buffer_aud = 0.0;
        input.debt_recycle = Some(recycle);

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        assert_eq!(recycle.summary.draw_count, 3);
    }

    #[test]
    fn saved_data_from_monthly_only_era_still_loads() {
        let json = r#"{
            "enabled": true,
            "mortgage_id": 1,
            "monthly_redraw_aud": 1500.0,
            "emergency_buffer_aud": 10000.0
        }"#;
        let parsed: DebtRecycleInput = serde_json::from_str(json).unwrap();
        assert_relative_eq!(parsed.redraw_amount_aud, 1_500.0);
        assert_eq!(parsed.redraw_cadence, RedrawCadence::Monthly);
    }

    #[test]
    fn interest_is_apportioned_pro_rata() {
        let mut input = base_input();
        input.debt_recycle = Some(recycle_input(input.mortgages[0].id));

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        assert!(recycle.summary.total_deductible_interest > 0.0);
        assert!(
            recycle.summary.total_deductible_interest
                < out.portfolio_totals.projected_total_interest
        );
        for period in &recycle.periods {
            assert!(period.deductible_interest >= 0.0);
        }
    }

    #[test]
    fn principal_repayments_contaminate_recycled_balance() {
        let mut input = base_input();
        input.debt_recycle = Some(recycle_input(input.mortgages[0].id));

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        assert!(recycle.summary.recycled_principal_repaid > 0.0);
        assert!(
            recycle.summary.ending_recycled_debt_balance
                < recycle.summary.total_drawn - recycle.summary.recycled_principal_repaid + 0.01
        );
    }

    #[test]
    fn dividends_and_franking_increase_offset() {
        let mut input = base_input();
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.emergency_buffer_aud = 1_000_000_000.0;
        recycle.dividend_yield_percent = 12.0;
        recycle.starting_investment_aud = 100_000.0;
        input.debt_recycle = Some(recycle);

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let offset_start = out
            .chart_series
            .offset_balance
            .first()
            .copied()
            .unwrap_or(0.0);
        let offset_next = out
            .chart_series
            .offset_balance
            .get(1)
            .copied()
            .unwrap_or(0.0);
        assert!(offset_next > offset_start);
    }

    #[test]
    fn validation_requires_positive_monthly_redraw() {
        let mut input = base_input();
        let mut recycle = recycle_input(input.mortgages[0].id);
        recycle.redraw_amount_aud = 0.0;
        input.debt_recycle = Some(recycle);

        let issues = validate_portfolio_input(&input);
        assert!(issues
            .iter()
            .any(|i| i.field == "debt_recycle.redraw_amount_aud"));
    }

    #[test]
    fn strategy_skipped_without_variable_owner_occupied_split() {
        let mut input = base_input();
        input.mortgages[0].splits[0].rate_type = RateType::Fixed;
        input.debt_recycle = Some(recycle_input(input.mortgages[0].id));

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        assert!(out.debt_recycle.is_none());
        assert!(out
            .warnings
            .iter()
            .any(|w| w.contains("no variable owner-occupied split")));
    }
}
