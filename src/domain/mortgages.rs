use serde::{Deserialize, Serialize};

#[cfg(target_arch = "wasm32")]
use crate::domain::calculator::calculate_income;
#[cfg(target_arch = "wasm32")]
use crate::domain::tax_rules::TaxRules;
#[cfg(target_arch = "wasm32")]
use crate::domain::types::CalculatorInput;
use crate::domain::types::{PayFrequency, ValidationIssue};

const MAX_MORTGAGES: usize = 10;
const MAX_SPLITS_PER_MORTGAGE: usize = 10;

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
pub struct DebtRecycleInput {
    pub enabled: bool,
    pub mortgage_id: u32,
    pub trigger_target_aud: f64,
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
            trigger_target_aud: 50_000.0,
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
    pub draw_amount: f64,
    pub new_split_id: Option<u32>,
    pub investment_value: f64,
    pub dividend_cash: f64,
    pub franking_credit: f64,
    pub offset_before: f64,
    pub offset_after: f64,
    pub recycled_debt_balance: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebtRecycleSummary {
    pub total_drawn: f64,
    pub draw_count: usize,
    pub ending_investment_value: f64,
    pub total_dividends: f64,
    pub total_franking_credits: f64,
    pub ending_recycled_debt_balance: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct DebtRecycleOutput {
    pub summary: DebtRecycleSummary,
    pub periods: Vec<DebtRecyclePeriod>,
    pub warnings: Vec<String>,
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

#[derive(Clone, Debug, PartialEq)]
pub enum MortgageValidationError {
    Validation(Vec<ValidationIssue>),
}

#[derive(Clone)]
struct WorkingSplit {
    balance: f64,
    rate_per_period: f64,
    io_periods: usize,
    repayment_type: LoanRepaymentType,
    loan_purpose: LoanPurpose,
    fixed_repayment: f64,
    is_recycled: bool,
}

#[derive(Clone)]
struct WorkingMortgage {
    id: u32,
    term_periods: usize,
    offset_balance: f64,
    splits: Vec<WorkingSplit>,
    next_split_id: u32,
}

#[derive(Clone, Copy)]
struct ProjectionDebtRecycleConfig {
    mortgage_index: usize,
    trigger_target: f64,
    emergency_buffer: f64,
    growth_rate_per_period: f64,
    dividend_rate_per_period: f64,
    franking_multiplier: f64,
    rate_per_period: f64,
    starting_investment: f64,
}

struct ProjectionDebtRecycleState {
    config: ProjectionDebtRecycleConfig,
    periods: Vec<DebtRecyclePeriod>,
    total_drawn: f64,
    draw_count: usize,
    total_dividends: f64,
    total_franking_credits: f64,
    investment_value: f64,
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

#[cfg(target_arch = "wasm32")]
pub fn load_income_context_from_saved_input(raw: &str) -> Option<IncomeContext> {
    let parsed = serde_json::from_str::<CalculatorInput>(raw).ok()?;
    let output = calculate_income(&parsed, &TaxRules::fy_2025_26_resident()).ok()?;
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
                    rate_per_period,
                    io_periods,
                    repayment_type: split.repayment_type,
                    loan_purpose: split.loan_purpose,
                    fixed_repayment,
                    is_recycled: false,
                }
            })
            .collect::<Vec<_>>();

        let next_split_id = mortgage.splits.iter().map(|s| s.id).max().unwrap_or(0) + 1;
        working.push(WorkingMortgage {
            id: mortgage.id,
            term_periods,
            offset_balance: mortgage.offset_balance,
            splits,
            next_split_id,
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

    let selected_split = mortgage_input
        .splits
        .iter()
        .find(|s| s.rate_type == RateType::Variable)
        .or_else(|| mortgage_input.splits.first());

    let Some(rate_source) = selected_split else {
        warnings
            .push("Debt recycle strategy skipped: selected mortgage has no splits.".to_string());
        return None;
    };

    if rate_source.rate_type != RateType::Variable {
        warnings.push(format!(
            "Debt recycle strategy for {} could not find a variable split; using first split rate {:.2}%.",
            mortgage_input.name, rate_source.annual_rate_percent
        ));
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
        trigger_target: debt_recycle.trigger_target_aud,
        emergency_buffer: debt_recycle.emergency_buffer_aud,
        growth_rate_per_period,
        dividend_rate_per_period,
        franking_multiplier,
        rate_per_period: rate_source.annual_rate_percent / 100.0 / periods_per_year,
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
        investment_value: config.starting_investment,
        warnings: Vec::new(),
    });

    period_months.push(0.0);
    balances.push(total_remaining_balance(&working));
    cumulative_repayment_series.push(0.0);
    cumulative_interest_series.push(0.0);
    offset_series.push(working.iter().map(|m| m.offset_balance).sum());

    for period in 1..=max_periods {
        for wm in &mut working {
            wm.offset_balance += offset_top_up_per_period;
        }

        if let Some(state) = &mut debt_recycle_state {
            let idx = state.config.mortgage_index;
            if let Some(target_mortgage) = working.get_mut(idx) {
                let dividend_cash =
                    (state.investment_value * state.config.dividend_rate_per_period).max(0.0);
                let franking_credit = (dividend_cash * state.config.franking_multiplier).max(0.0);
                target_mortgage.offset_balance += dividend_cash + franking_credit;

                let offset_before = target_mortgage.offset_balance;
                let mut draw_amount = 0.0;
                let mut new_split_id = None;

                if target_mortgage.offset_balance >= state.config.trigger_target {
                    draw_amount =
                        (target_mortgage.offset_balance - state.config.emergency_buffer).max(0.0);
                    if draw_amount > 0.0 {
                        target_mortgage.offset_balance -= draw_amount;
                        let shifted = shift_owner_occupied_debt(target_mortgage, draw_amount);
                        if shifted <= 0.0 {
                            continue;
                        }
                        let split_id = target_mortgage.next_split_id;
                        target_mortgage.next_split_id =
                            target_mortgage.next_split_id.saturating_add(1);
                        target_mortgage.splits.push(WorkingSplit {
                            balance: shifted,
                            rate_per_period: state.config.rate_per_period,
                            io_periods: usize::MAX,
                            repayment_type: LoanRepaymentType::InterestOnlyThenPrincipalAndInterest,
                            loan_purpose: LoanPurpose::Investment,
                            fixed_repayment: 0.0,
                            is_recycled: true,
                        });
                        state.investment_value += shifted;
                        state.total_drawn += shifted;
                        state.draw_count += 1;
                        new_split_id = Some(split_id);
                        if shifted < draw_amount {
                            state.warnings.push(format!(
                                "Period {period}: recycle draw capped at {} due to available owner-occupied debt.",
                                shifted.round()
                            ));
                        }
                        draw_amount = shifted;
                    }
                }

                state.total_dividends += dividend_cash;
                state.total_franking_credits += franking_credit;
                state.investment_value *= 1.0 + state.config.growth_rate_per_period;
                let offset_after = target_mortgage.offset_balance;
                let recycled_debt_balance = recycled_debt_balance(&working);

                state.periods.push(DebtRecyclePeriod {
                    period_index: period,
                    draw_amount,
                    new_split_id,
                    investment_value: state.investment_value,
                    dividend_cash,
                    franking_credit,
                    offset_before,
                    offset_after,
                    recycled_debt_balance,
                });
            } else {
                state.warnings.push(
                    "Debt recycle strategy stopped because selected mortgage became unavailable."
                        .to_string(),
                );
            }
        }

        for wm in &mut working {
            if owner_occupied_non_recycled_balance(wm) <= 1e-9 {
                for split in wm.splits.iter_mut().filter(|s| s.is_recycled) {
                    // Once non-deductible debt is gone, recycled debt starts amortizing.
                    split.io_periods = period.saturating_sub(1);
                }
            }
        }

        let opening_balance = total_remaining_balance(&working);
        let mut period_interest = 0.0;
        let mut period_principal = 0.0;
        let mut period_repayment = 0.0;

        for wm in &mut working {
            if period > wm.term_periods {
                continue;
            }

            let total_split_balance = wm.splits.iter().map(|s| s.balance).sum::<f64>();
            if total_split_balance <= 0.0 {
                continue;
            }

            let effective_offset = wm.offset_balance.min(total_split_balance);

            for split in &mut wm.splits {
                if split.balance <= 0.0 {
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
                split.balance -= principal;

                period_interest += interest;
                period_principal += principal;
                period_repayment += interest + principal;
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
        .filter(|s| s.is_recycled)
        .map(|s| s.balance)
        .sum()
}

fn shift_owner_occupied_debt(mortgage: &mut WorkingMortgage, requested: f64) -> f64 {
    let mut remaining = requested.max(0.0);
    if remaining <= 0.0 {
        return 0.0;
    }

    // Shift only owner-occupied debt (non-deductible debt recycling).
    for split in mortgage.splits.iter_mut().filter(|s| {
        !s.is_recycled && s.loan_purpose == LoanPurpose::OwnerOccupied && s.balance > 0.0
    }) {
        let taken = split.balance.min(remaining);
        split.balance -= taken;
        remaining -= taken;
        if remaining <= 1e-9 {
            break;
        }
    }

    requested.max(0.0) - remaining.max(0.0)
}

fn owner_occupied_non_recycled_balance(mortgage: &WorkingMortgage) -> f64 {
    mortgage
        .splits
        .iter()
        .filter(|s| !s.is_recycled && s.loan_purpose == LoanPurpose::OwnerOccupied)
        .map(|s| s.balance)
        .sum()
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
        if debt_recycle.trigger_target_aud < 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.trigger_target_aud",
                message: "Debt recycle trigger target must be zero or greater.".to_string(),
            });
        }
        if debt_recycle.emergency_buffer_aud < 0.0 {
            issues.push(ValidationIssue {
                field: "debt_recycle.emergency_buffer_aud",
                message: "Debt recycle emergency buffer must be zero or greater.".to_string(),
            });
        }
        if debt_recycle.trigger_target_aud < debt_recycle.emergency_buffer_aud {
            issues.push(ValidationIssue {
                field: "debt_recycle.trigger_target_aud",
                message:
                    "Debt recycle trigger target must be greater than or equal to emergency buffer."
                        .to_string(),
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

    #[test]
    fn debt_recycle_creates_draw_splits() {
        let mut input = base_input();
        input.debt_recycle = Some(DebtRecycleInput {
            enabled: true,
            mortgage_id: input.mortgages[0].id,
            trigger_target_aud: 10_000.0,
            emergency_buffer_aud: 5_000.0,
            growth_rate_percent: 0.0,
            dividend_yield_percent: 0.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 0.0,
        });

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let recycle = out.debt_recycle.expect("expected debt recycle output");
        assert!(recycle.summary.draw_count > 0);
        assert_eq!(
            recycle
                .periods
                .iter()
                .filter(|p| p.new_split_id.is_some())
                .count(),
            recycle.summary.draw_count
        );
    }

    #[test]
    fn debt_recycle_dividends_and_franking_increase_offset() {
        let mut input = base_input();
        input.debt_recycle = Some(DebtRecycleInput {
            enabled: true,
            mortgage_id: input.mortgages[0].id,
            trigger_target_aud: 1_000_000_000.0,
            emergency_buffer_aud: 20_000.0,
            growth_rate_percent: 0.0,
            dividend_yield_percent: 12.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 100_000.0,
        });

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
    fn debt_recycle_validation_requires_target_at_least_buffer() {
        let mut input = base_input();
        input.debt_recycle = Some(DebtRecycleInput {
            enabled: true,
            mortgage_id: input.mortgages[0].id,
            trigger_target_aud: 10_000.0,
            emergency_buffer_aud: 20_000.0,
            growth_rate_percent: 6.0,
            dividend_yield_percent: 4.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 0.0,
        });

        let issues = validate_portfolio_input(&input);
        assert!(issues
            .iter()
            .any(|i| i.field == "debt_recycle.trigger_target_aud"));
    }

    #[test]
    fn debt_recycle_keeps_total_debt_neutral_on_shift() {
        let mut input = base_input();
        input.mortgages[0].splits[0].repayment_type =
            LoanRepaymentType::InterestOnlyThenPrincipalAndInterest;
        input.mortgages[0].splits[0].interest_only_years = 29.0;
        input.debt_recycle = Some(DebtRecycleInput {
            enabled: true,
            mortgage_id: input.mortgages[0].id,
            trigger_target_aud: 10_000.0,
            emergency_buffer_aud: 5_000.0,
            growth_rate_percent: 0.0,
            dividend_yield_percent: 0.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 0.0,
        });

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
    fn recycled_debt_switches_to_pi_after_owner_occupied_is_cleared() {
        let mut input = base_input();
        input.mortgages[0].offset_balance = 600_000.0;
        input.mortgages[0].splits[0].repayment_type =
            LoanRepaymentType::InterestOnlyThenPrincipalAndInterest;
        input.mortgages[0].splits[0].interest_only_years = 29.0;
        input.debt_recycle = Some(DebtRecycleInput {
            enabled: true,
            mortgage_id: input.mortgages[0].id,
            trigger_target_aud: 1_000.0,
            emergency_buffer_aud: 0.0,
            growth_rate_percent: 0.0,
            dividend_yield_percent: 0.0,
            franking_percent: 100.0,
            company_tax_rate_percent: 30.0,
            starting_investment_aud: 0.0,
        });

        let out = calculate_mortgage_portfolio(&input, None).unwrap();
        let first_row = out.amortization_rows.first().expect("first row expected");
        assert!(
            first_row.principal > 0.0,
            "recycled split should move to P&I once owner-occupied debt is cleared"
        );
    }
}
