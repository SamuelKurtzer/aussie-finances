use crate::domain::tax_rules::{HelpBracket, MlsTier, TaxBracket, TaxRules};
use crate::domain::types::{CalculatorError, CalculatorInput, CalculatorOutput};
use crate::domain::validation::validate_input;

pub fn calculate_income(
    input: &CalculatorInput,
    rules: &TaxRules,
) -> Result<CalculatorOutput, CalculatorError> {
    let issues = validate_input(input);
    if !issues.is_empty() {
        return Err(CalculatorError::Validation(issues));
    }

    let gross_income_annual = input.gross_income_annual;
    let gross_base_for_tax = if input.includes_super {
        gross_income_annual / (1.0 + input.super_rate_percent / 100.0)
    } else {
        gross_income_annual
    };

    let taxable_income_annual =
        (gross_base_for_tax - input.salary_sacrifice_annual - input.deductions_annual).max(0.0);

    let income_tax_annual =
        compute_progressive_tax(taxable_income_annual, &rules.resident_tax_brackets);
    let medicare_levy_annual = compute_medicare_levy(
        taxable_income_annual,
        rules.medicare_levy_low_income_threshold,
        rules.medicare_levy_rate,
    );

    let mls_income = input
        .mls_income_for_surcharge_annual
        .unwrap_or(taxable_income_annual + input.reportable_fringe_benefits_annual);
    let medicare_levy_surcharge_annual = compute_mls(
        mls_income,
        input.has_private_hospital_cover,
        &rules.mls_tiers,
    );

    let help_repayment_annual = if input.has_help_debt {
        compute_help_repayment(taxable_income_annual, &rules.help_brackets)
    } else {
        0.0
    };

    let total_withheld_annual = income_tax_annual
        + medicare_levy_annual
        + medicare_levy_surcharge_annual
        + help_repayment_annual;

    let net_income_annual = (gross_base_for_tax - total_withheld_annual).max(0.0);
    let period_divisor = input.pay_frequency.periods_per_year();

    let effective_tax_rate_percent = if gross_base_for_tax > 0.0 {
        (total_withheld_annual / gross_base_for_tax) * 100.0
    } else {
        0.0
    };

    let mut assumptions = vec![
        format!("Tax rules: {}.", rules.name),
        "Resident-only model for MVP (non-resident and WHM not yet supported).".to_string(),
        "Income tax offsets/rebates are excluded in this release.".to_string(),
        "Estimate only: not personal financial advice.".to_string(),
    ];

    if input.includes_super {
        assumptions.push("Gross input is treated as total package including super and converted to pre-tax salary base.".to_string());
    }

    Ok(CalculatorOutput {
        gross_income_annual: gross_base_for_tax,
        gross_income_period: gross_base_for_tax / period_divisor,
        taxable_income_annual,
        income_tax_annual,
        medicare_levy_annual,
        medicare_levy_surcharge_annual,
        help_repayment_annual,
        total_withheld_annual,
        net_income_annual,
        net_income_period: net_income_annual / period_divisor,
        effective_tax_rate_percent,
        pay_frequency: input.pay_frequency,
        assumptions,
    })
}

fn compute_progressive_tax(income: f64, brackets: &[TaxBracket]) -> f64 {
    let mut tax = 0.0;
    for bracket in brackets {
        let upper = bracket.upper_bound.unwrap_or(f64::MAX);
        if income > bracket.lower_bound {
            let taxable_at_rate = (income.min(upper) - bracket.lower_bound).max(0.0);
            tax += taxable_at_rate * bracket.rate;
        }
    }
    tax
}

fn compute_medicare_levy(income: f64, threshold: f64, rate: f64) -> f64 {
    if income <= threshold {
        0.0
    } else {
        income * rate
    }
}

fn compute_help_repayment(income: f64, brackets: &[HelpBracket]) -> f64 {
    let mut selected_rate = 0.0;
    for bracket in brackets {
        let upper = bracket.upper_bound.unwrap_or(f64::MAX);
        if income >= bracket.lower_bound && income < upper {
            selected_rate = bracket.rate;
            break;
        }
        if bracket.upper_bound.is_none() && income >= bracket.lower_bound {
            selected_rate = bracket.rate;
        }
    }
    income * selected_rate
}

fn compute_mls(income_for_mls: f64, has_private_cover: bool, tiers: &[MlsTier]) -> f64 {
    if has_private_cover {
        return 0.0;
    }
    let mut rate = 0.0;
    for tier in tiers {
        let upper = tier.upper_bound.unwrap_or(f64::MAX);
        if income_for_mls >= tier.lower_bound && income_for_mls < upper {
            rate = tier.rate;
            break;
        }
        if tier.upper_bound.is_none() && income_for_mls >= tier.lower_bound {
            rate = tier.rate;
        }
    }
    income_for_mls * rate
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::calculate_income;
    use crate::domain::tax_rules::TaxRules;
    use crate::domain::types::{CalculatorInput, PayFrequency};

    #[test]
    fn bracket_boundary_tax_works() {
        let rules = TaxRules::fy_2025_26_resident();
        let mut input = CalculatorInput::default();
        input.gross_income_annual = 45_000.0;
        input.pay_frequency = PayFrequency::Annually;

        let output = calculate_income(&input, &rules).unwrap();
        assert_relative_eq!(output.income_tax_annual, 4_288.0, epsilon = 0.1);
    }

    #[test]
    fn help_debt_applies_when_enabled() {
        let rules = TaxRules::fy_2025_26_resident();
        let mut input = CalculatorInput::default();
        input.gross_income_annual = 80_000.0;
        input.has_help_debt = true;

        let output = calculate_income(&input, &rules).unwrap();
        assert!(output.help_repayment_annual > 0.0);
    }

    #[test]
    fn help_debt_not_overstated_at_100k() {
        let rules = TaxRules::fy_2025_26_resident();
        let mut input = CalculatorInput::default();
        input.gross_income_annual = 100_000.0;
        input.has_help_debt = true;

        let output = calculate_income(&input, &rules).unwrap();
        assert_relative_eq!(output.help_repayment_annual, 5_500.0, epsilon = 0.1);
    }

    #[test]
    fn mls_zero_when_private_cover() {
        let rules = TaxRules::fy_2025_26_resident();
        let mut input = CalculatorInput::default();
        input.gross_income_annual = 160_000.0;
        input.has_private_hospital_cover = true;

        let output = calculate_income(&input, &rules).unwrap();
        assert_relative_eq!(output.medicare_levy_surcharge_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn frequency_conversion_consistent() {
        let rules = TaxRules::fy_2025_26_resident();
        let mut input = CalculatorInput::default();
        input.gross_income_annual = 104_000.0;
        input.pay_frequency = PayFrequency::Weekly;

        let output = calculate_income(&input, &rules).unwrap();
        assert_relative_eq!(output.gross_income_period, 2000.0, epsilon = 0.01);
    }

    #[test]
    fn deductions_reduce_taxable_income() {
        let rules = TaxRules::fy_2025_26_resident();
        let mut baseline = CalculatorInput::default();
        baseline.gross_income_annual = 100_000.0;
        let with_deductions = CalculatorInput {
            deductions_annual: 5_000.0,
            ..baseline.clone()
        };

        let a = calculate_income(&baseline, &rules).unwrap();
        let b = calculate_income(&with_deductions, &rules).unwrap();
        assert!(b.taxable_income_annual < a.taxable_income_annual);
        assert!(b.total_withheld_annual < a.total_withheld_annual);
    }
}
