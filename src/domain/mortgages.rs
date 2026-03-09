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
    pub mortgages: Vec<MortgageInput>,
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
    fixed_repayment: f64,
}

#[derive(Clone)]
struct WorkingMortgage {
    term_periods: usize,
    offset_balance: f64,
    splits: Vec<WorkingSplit>,
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
                let io_periods = (split.interest_only_years * periods_per_year as f64).round() as usize;
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
                    fixed_repayment,
                }
            })
            .collect::<Vec<_>>();

        working.push(WorkingMortgage {
            term_periods,
            offset_balance: mortgage.offset_balance,
            splits,
        });
    }

    let max_periods = working.iter().map(|w| w.term_periods).max().unwrap_or(0);
    let baseline_projection = run_projection(
        working.clone(),
        max_periods,
        input.offset_top_up_per_period,
        input.repayment_cadence,
    );
    let mut worst_working = working;
    for wm in &mut worst_working {
        wm.offset_balance = 0.0;
    }
    let worst_projection = run_projection(worst_working, max_periods, 0.0, input.repayment_cadence);

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
        repayment_to_income_percent,
        warnings,
    })
}

fn run_projection(
    mut working: Vec<WorkingMortgage>,
    max_periods: usize,
    offset_top_up_per_period: f64,
    cadence: RepaymentCadence,
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

    period_months.push(0.0);
    balances.push(total_remaining_balance(&working));
    cumulative_repayment_series.push(0.0);
    cumulative_interest_series.push(0.0);
    offset_series.push(working.iter().map(|m| m.offset_balance).sum());

    for period in 1..=max_periods {
        for wm in &mut working {
            wm.offset_balance += offset_top_up_per_period;
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
                    LoanRepaymentType::PrincipalAndInterest => split
                        .fixed_repayment
                        .min(interest + split.balance),
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
    }
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
}
