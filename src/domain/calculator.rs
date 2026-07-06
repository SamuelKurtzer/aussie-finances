use crate::domain::tax_rules::{Bracket, HelpSystem, TaxRules};
use crate::domain::types::{
    BracketLine, CalculatorError, CalculatorInput, CalculatorOutput, MedicareExemption, Residency,
};
use crate::domain::validation::validate_input;

pub fn calculate_income(
    input: &CalculatorInput,
    rules: &TaxRules,
) -> Result<CalculatorOutput, CalculatorError> {
    let issues = validate_input(input);
    if !issues.is_empty() {
        return Err(CalculatorError::Validation(issues));
    }

    let annual_salary_input = input.annual_salary();
    let annual_salary = if input.includes_super {
        annual_salary_input / (1.0 + input.super_rate_percent / 100.0)
    } else {
        annual_salary_input
    };

    let gross_base_for_tax = annual_salary + input.bonus_annual + input.overtime_annual;
    // SG is payable on ordinary time earnings (salary + bonus) but not overtime.
    let sg_base = annual_salary + input.bonus_annual;

    let taxable_income_annual = (gross_base_for_tax
        - input.salary_sacrifice_annual
        - input.extra_super_annual
        - input.deductions_annual)
        .max(0.0);

    let brackets = tax_brackets_for(input.residency, rules);
    let income_tax_annual = compute_progressive_tax(taxable_income_annual, brackets);
    let bracket_breakdown = compute_bracket_breakdown(taxable_income_annual, brackets);
    let marginal_rate_percent = find_bracket_rate(taxable_income_annual, brackets) * 100.0;

    let is_resident = input.residency == Residency::Resident;

    let lito_annual = if is_resident {
        compute_lito(taxable_income_annual, rules).min(income_tax_annual)
    } else {
        0.0
    };
    let sapto_annual = if is_resident && input.is_sapto_eligible {
        compute_sapto(taxable_income_annual, rules).min(income_tax_annual - lito_annual)
    } else {
        0.0
    };

    let medicare_levy_annual = if is_resident {
        compute_medicare_levy(taxable_income_annual, input, rules)
    } else {
        0.0
    };

    let mls_income = input
        .mls_income_for_surcharge_annual
        .unwrap_or(taxable_income_annual + input.reportable_fringe_benefits_annual);
    let medicare_levy_surcharge_annual = if is_resident {
        compute_mls(mls_income, input, rules)
    } else {
        0.0
    };

    let help_repayment_annual = if input.has_help_debt {
        compute_help_repayment(taxable_income_annual, rules)
    } else {
        0.0
    };

    let super_guarantee_annual = sg_base * input.super_rate_percent / 100.0;
    let concessional_contributions_annual =
        super_guarantee_annual + input.salary_sacrifice_annual + input.extra_super_annual;

    let division_293_income = taxable_income_annual + concessional_contributions_annual;
    let division_293_annual = if division_293_income > rules.division_293_threshold {
        let excess = division_293_income - rules.division_293_threshold;
        rules.division_293_rate * excess.min(concessional_contributions_annual)
    } else {
        0.0
    };

    let total_withheld_annual = (income_tax_annual - lito_annual - sapto_annual)
        + medicare_levy_annual
        + medicare_levy_surcharge_annual
        + help_repayment_annual;

    let net_income_annual = (gross_base_for_tax
        - input.salary_sacrifice_annual
        - input.extra_super_annual
        - total_withheld_annual)
        .max(0.0);
    let period_divisor = input.pay_frequency.periods_per_year();

    let effective_tax_rate_percent = if gross_base_for_tax > 0.0 {
        (total_withheld_annual / gross_base_for_tax) * 100.0
    } else {
        0.0
    };

    let mut warnings = Vec::new();
    if concessional_contributions_annual > rules.concessional_contributions_cap {
        warnings.push(format!(
            "Concessional super contributions (${:.0}) exceed the ${:.0} cap; the excess is taxed at your marginal rate.",
            concessional_contributions_annual, rules.concessional_contributions_cap
        ));
    }

    let mut assumptions = vec![
        format!("Tax rules: {} ({}).", rules.name, input.residency.as_str()),
        "Study loan repayments cover HELP/HECS, VET, SSL, TSL, and SFSS at unified STSL rates, computed on taxable income.".to_string(),
        "SG super is paid on salary and bonus but not overtime.".to_string(),
        "Salary sacrifice and extra super are treated as concessional contributions that reduce taxable income and take-home pay.".to_string(),
        "Family Medicare reduction is an estimate against combined family income; SAPTO uses single rates.".to_string(),
        "Division 293 is an estimate, assessed separately against your super fund; it is not included in Total Withheld.".to_string(),
        "Estimate only: not personal financial advice.".to_string(),
    ];

    if input.includes_super {
        assumptions.push("Salary input is treated as a package including super and converted to a pre-tax base (bonus and overtime are treated as super-exclusive).".to_string());
    }

    Ok(CalculatorOutput {
        gross_income_annual: gross_base_for_tax,
        gross_income_period: gross_base_for_tax / period_divisor,
        taxable_income_annual,
        income_tax_annual,
        lito_annual,
        sapto_annual,
        medicare_levy_annual,
        medicare_levy_surcharge_annual,
        help_repayment_annual,
        total_withheld_annual,
        net_income_annual,
        net_income_period: net_income_annual / period_divisor,
        effective_tax_rate_percent,
        marginal_rate_percent,
        super_guarantee_annual,
        concessional_contributions_annual,
        division_293_annual,
        bracket_breakdown,
        pay_frequency: input.pay_frequency,
        warnings,
        assumptions,
    })
}

fn tax_brackets_for(residency: Residency, rules: &TaxRules) -> &[Bracket] {
    match residency {
        Residency::Resident => &rules.resident_tax_brackets,
        Residency::NonResident => &rules.non_resident_tax_brackets,
        Residency::WorkingHolidayMaker => &rules.whm_tax_brackets,
    }
}

fn compute_progressive_tax(income: f64, brackets: &[Bracket]) -> f64 {
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

fn compute_bracket_breakdown(income: f64, brackets: &[Bracket]) -> Vec<BracketLine> {
    brackets
        .iter()
        .map(|bracket| {
            let upper = bracket.upper_bound.unwrap_or(f64::MAX);
            let taxable_at_rate = (income.min(upper) - bracket.lower_bound).max(0.0);
            BracketLine {
                lower_bound: bracket.lower_bound,
                upper_bound: bracket.upper_bound,
                rate: bracket.rate,
                tax_amount: taxable_at_rate * bracket.rate,
            }
        })
        .collect()
}

fn compute_lito(taxable_income: f64, rules: &TaxRules) -> f64 {
    let taper_one = rules.lito_taper_one_rate
        * (taxable_income.min(rules.lito_taper_two_start) - rules.lito_taper_one_start).max(0.0);
    let taper_two =
        rules.lito_taper_two_rate * (taxable_income - rules.lito_taper_two_start).max(0.0);
    (rules.lito_max_offset - taper_one - taper_two).max(0.0)
}

fn compute_sapto(taxable_income: f64, rules: &TaxRules) -> f64 {
    (rules.sapto_max_offset
        - rules.sapto_taper_rate * (taxable_income - rules.sapto_taper_start).max(0.0))
    .max(0.0)
}

fn compute_medicare_levy(taxable: f64, input: &CalculatorInput, rules: &TaxRules) -> f64 {
    let exemption_multiplier = match input.medicare_exemption {
        MedicareExemption::Full => return 0.0,
        MedicareExemption::Half => 0.5,
        MedicareExemption::None => 1.0,
    };

    let full_levy = taxable * rules.medicare_levy_rate;
    let is_family = input.has_family || input.dependants > 0;

    let reduced_levy = if is_family {
        let base_threshold = if input.is_sapto_eligible {
            rules.medicare_levy_sapto_family_threshold
        } else {
            rules.medicare_levy_family_threshold
        };
        let threshold =
            base_threshold + rules.medicare_levy_child_increment * input.dependants as f64;
        let family_income = input.family_income_annual.unwrap_or(taxable).max(taxable);
        (rules.medicare_levy_phase_in_rate * (family_income - threshold)).max(0.0)
    } else {
        let threshold = if input.is_sapto_eligible {
            rules.medicare_levy_sapto_threshold
        } else {
            rules.medicare_levy_low_income_threshold
        };
        (rules.medicare_levy_phase_in_rate * (taxable - threshold)).max(0.0)
    };

    full_levy.min(reduced_levy) * exemption_multiplier
}

fn find_bracket_rate(income: f64, brackets: &[Bracket]) -> f64 {
    for bracket in brackets {
        let upper = bracket.upper_bound.unwrap_or(f64::MAX);
        if income >= bracket.lower_bound && income < upper {
            return bracket.rate;
        }
        if bracket.upper_bound.is_none() && income >= bracket.lower_bound {
            return bracket.rate;
        }
    }
    0.0
}

fn compute_help_repayment(income: f64, rules: &TaxRules) -> f64 {
    match rules.help_system {
        HelpSystem::AverageRate => income * find_bracket_rate(income, &rules.help_brackets),
        HelpSystem::Marginal => compute_progressive_tax(income, &rules.help_brackets)
            .min(income * rules.help_repayment_cap_rate),
    }
}

fn compute_mls(mls_income: f64, input: &CalculatorInput, rules: &TaxRules) -> f64 {
    if input.has_private_hospital_cover {
        return 0.0;
    }
    let is_family = input.has_family || input.dependants > 0;
    let (tiers, tier_test_income) = if is_family {
        // The family threshold rises per dependent child after the first,
        // modelled by reducing the tested income.
        let child_adjust =
            rules.mls_family_child_increment * input.dependants.saturating_sub(1) as f64;
        let family_income = input.family_income_annual.unwrap_or(mls_income).max(mls_income);
        (&rules.mls_family_tiers, family_income - child_adjust)
    } else {
        (&rules.mls_tiers, mls_income)
    };
    mls_income * find_bracket_rate(tier_test_income, tiers)
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::calculate_income;
    use crate::domain::tax_rules::TaxRules;
    use crate::domain::types::{
        CalculatorInput, FinancialYear, IncomeUnit, MedicareExemption, PayFrequency, Residency,
    };

    fn rules() -> TaxRules {
        TaxRules::fy_2025_26()
    }

    #[test]
    fn bracket_boundary_tax_works() {
        let mut input = CalculatorInput::default();
        input.income_amount = 45_000.0;
        input.pay_frequency = PayFrequency::Annually;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.income_tax_annual, 4_288.0, epsilon = 0.1);
    }

    #[test]
    fn hourly_income_annualises() {
        let mut input = CalculatorInput::default();
        input.income_amount = 50.0;
        input.income_unit = IncomeUnit::Hourly;
        input.hours_per_week = 38.0;

        // 50 * 38 * 52 = 98,800
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.gross_income_annual, 98_800.0, epsilon = 0.1);
    }

    #[test]
    fn daily_income_annualises() {
        let mut input = CalculatorInput::default();
        input.income_amount = 400.0;
        input.income_unit = IncomeUnit::Daily;
        input.days_per_week = 5.0;

        // 400 * 5 * 52 = 104,000
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.gross_income_annual, 104_000.0, epsilon = 0.1);
    }

    #[test]
    fn weekly_income_annualises() {
        let mut input = CalculatorInput::default();
        input.income_amount = 2_000.0;
        input.income_unit = IncomeUnit::Weekly;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.gross_income_annual, 104_000.0, epsilon = 0.1);
    }

    #[test]
    fn bonus_taxed_and_attracts_sg() {
        let mut baseline = CalculatorInput::default();
        baseline.income_amount = 100_000.0;
        let with_bonus = CalculatorInput {
            bonus_annual: 10_000.0,
            ..baseline.clone()
        };

        let a = calculate_income(&baseline, &rules()).unwrap();
        let b = calculate_income(&with_bonus, &rules()).unwrap();
        assert_relative_eq!(b.taxable_income_annual, 110_000.0, epsilon = 0.1);
        assert_relative_eq!(
            b.super_guarantee_annual,
            a.super_guarantee_annual + 1_200.0,
            epsilon = 0.1
        );
    }

    #[test]
    fn overtime_taxed_but_no_sg() {
        let mut baseline = CalculatorInput::default();
        baseline.income_amount = 100_000.0;
        let with_overtime = CalculatorInput {
            overtime_annual: 10_000.0,
            ..baseline.clone()
        };

        let a = calculate_income(&baseline, &rules()).unwrap();
        let b = calculate_income(&with_overtime, &rules()).unwrap();
        assert_relative_eq!(b.taxable_income_annual, 110_000.0, epsilon = 0.1);
        assert_relative_eq!(
            b.super_guarantee_annual,
            a.super_guarantee_annual,
            epsilon = 0.1
        );
    }

    #[test]
    fn fy_2024_25_uses_average_rate_help() {
        let mut input = CalculatorInput::default();
        input.income_amount = 100_000.0;
        input.financial_year = FinancialYear::Fy2024_25;
        input.has_help_debt = true;

        // Old system: 5.5% average rate on the whole $100k.
        let output =
            calculate_income(&input, &TaxRules::for_year(input.financial_year)).unwrap();
        assert_relative_eq!(output.help_repayment_annual, 5_500.0, epsilon = 0.1);
    }

    #[test]
    fn help_repayment_zero_below_threshold() {
        let mut input = CalculatorInput::default();
        input.income_amount = 66_000.0;
        input.has_help_debt = true;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.help_repayment_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn help_marginal_at_100k() {
        let mut input = CalculatorInput::default();
        input.income_amount = 100_000.0;
        input.has_help_debt = true;

        // 15% of (100k - 67k) = 4,950 under the FY25-26 marginal system.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.help_repayment_annual, 4_950.0, epsilon = 0.1);
    }

    #[test]
    fn help_capped_at_ten_percent_of_income() {
        let mut input = CalculatorInput::default();
        input.income_amount = 250_000.0;
        input.has_help_debt = true;

        // Marginal: 8,700 + 17% * 125,000 = 29,950; cap: 10% * 250,000 = 25,000.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.help_repayment_annual, 25_000.0, epsilon = 0.1);
    }

    #[test]
    fn lito_full_at_low_income() {
        let mut input = CalculatorInput::default();
        input.income_amount = 30_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.lito_annual, 700.0, epsilon = 0.01);
    }

    #[test]
    fn lito_tapers_out_by_66667() {
        let mut input = CalculatorInput::default();
        input.income_amount = 70_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.lito_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn sapto_full_below_taper_start() {
        let mut input = CalculatorInput::default();
        input.income_amount = 32_000.0;
        input.is_sapto_eligible = true;

        // Income tax at 32k = 16% * (32,000 - 18,200) = 2,208; LITO 700 leaves 1,508.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.sapto_annual, 1_508.0, epsilon = 0.1);
    }

    #[test]
    fn sapto_tapers_to_zero() {
        let mut input = CalculatorInput::default();
        input.income_amount = 60_000.0;
        input.is_sapto_eligible = true;

        // 2,230 - 12.5% * (60,000 - 32,279) = negative → 0.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.sapto_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn sapto_raises_medicare_threshold() {
        let mut input = CalculatorInput::default();
        input.income_amount = 40_000.0;
        input.is_sapto_eligible = true;

        // Below the $44,267 senior threshold → no levy.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn medicare_levy_phases_in_above_threshold() {
        let mut input = CalculatorInput::default();
        input.income_amount = 30_000.0;

        // 10% of (30,000 - 28,011) = 198.90, less than 2% of 30,000 = 600.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_annual, 198.90, epsilon = 0.01);
    }

    #[test]
    fn medicare_levy_full_rate_at_high_income() {
        let mut input = CalculatorInput::default();
        input.income_amount = 90_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_annual, 1_800.0, epsilon = 0.01);
    }

    #[test]
    fn medicare_full_exemption_zeroes_levy() {
        let mut input = CalculatorInput::default();
        input.income_amount = 90_000.0;
        input.medicare_exemption = MedicareExemption::Full;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn medicare_half_exemption_halves_levy() {
        let mut input = CalculatorInput::default();
        input.income_amount = 90_000.0;
        input.medicare_exemption = MedicareExemption::Half;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_annual, 900.0, epsilon = 0.01);
    }

    #[test]
    fn family_threshold_reduces_levy_for_low_family_income() {
        let mut input = CalculatorInput::default();
        input.income_amount = 45_000.0;
        input.has_family = true;
        input.dependants = 2;
        input.family_income_annual = Some(50_000.0);

        // Family threshold: 47,238 + 2 * 4,338 = 55,914 > 50,000 → no levy.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn mls_uses_family_tiers_when_family() {
        let mut input = CalculatorInput::default();
        input.income_amount = 150_000.0;
        input.has_family = true;
        input.family_income_annual = Some(180_000.0);

        // Family income 180k < 202k family tier 1 → no surcharge,
        // even though 150k alone would hit the single tiers.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_surcharge_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn mls_single_tier_one_at_105k() {
        let mut input = CalculatorInput::default();
        input.income_amount = 105_000.0;

        // 1% of 105,000 (tier 1: 101k-118k in FY25-26).
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_surcharge_annual, 1_050.0, epsilon = 0.1);
    }

    #[test]
    fn mls_fy_2024_25_tier_starts_at_97k() {
        let mut input = CalculatorInput::default();
        input.income_amount = 99_000.0;
        input.financial_year = FinancialYear::Fy2024_25;

        let output =
            calculate_income(&input, &TaxRules::for_year(input.financial_year)).unwrap();
        assert_relative_eq!(output.medicare_levy_surcharge_annual, 990.0, epsilon = 0.1);
    }

    #[test]
    fn non_resident_flat_thirty_no_medicare_no_lito() {
        let mut input = CalculatorInput::default();
        input.income_amount = 100_000.0;
        input.residency = Residency::NonResident;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.income_tax_annual, 30_000.0, epsilon = 0.1);
        assert_relative_eq!(output.medicare_levy_annual, 0.0, epsilon = 0.01);
        assert_relative_eq!(output.lito_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn whm_fifteen_percent_first_bracket() {
        let mut input = CalculatorInput::default();
        input.income_amount = 45_000.0;
        input.residency = Residency::WorkingHolidayMaker;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.income_tax_annual, 6_750.0, epsilon = 0.1);
    }

    #[test]
    fn super_guarantee_reported() {
        let mut input = CalculatorInput::default();
        input.income_amount = 100_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.super_guarantee_annual, 12_000.0, epsilon = 0.1);
    }

    #[test]
    fn concessional_cap_warning_when_exceeded() {
        let mut input = CalculatorInput::default();
        input.income_amount = 200_000.0;
        input.salary_sacrifice_annual = 10_000.0;

        // SG 24,000 + 10,000 sacrifice = 34,000 > 30,000 cap.
        let output = calculate_income(&input, &rules()).unwrap();
        assert!(!output.warnings.is_empty());
    }

    #[test]
    fn division_293_applies_above_threshold() {
        let mut input = CalculatorInput::default();
        input.income_amount = 280_000.0;
        input.pay_frequency = PayFrequency::Annually;

        // Taxable 280k + SG 33.6k = 313.6k; excess 63.6k > contributions 33.6k,
        // so 15% applies to all 33.6k of contributions.
        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.division_293_annual, 5_040.0, epsilon = 0.1);
    }

    #[test]
    fn division_293_zero_below_threshold() {
        let mut input = CalculatorInput::default();
        input.income_amount = 150_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.division_293_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn marginal_rate_reported() {
        let mut input = CalculatorInput::default();
        input.income_amount = 100_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.marginal_rate_percent, 30.0, epsilon = 0.01);
    }

    #[test]
    fn extra_super_reduces_taxable_income() {
        let mut baseline = CalculatorInput::default();
        baseline.income_amount = 100_000.0;
        let with_extra = CalculatorInput {
            extra_super_annual: 5_000.0,
            ..baseline.clone()
        };

        let a = calculate_income(&baseline, &rules()).unwrap();
        let b = calculate_income(&with_extra, &rules()).unwrap();
        assert_relative_eq!(
            b.taxable_income_annual,
            a.taxable_income_annual - 5_000.0,
            epsilon = 0.01
        );
    }

    #[test]
    fn mls_zero_when_private_cover() {
        let mut input = CalculatorInput::default();
        input.income_amount = 160_000.0;
        input.has_private_hospital_cover = true;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.medicare_levy_surcharge_annual, 0.0, epsilon = 0.01);
    }

    #[test]
    fn frequency_conversion_consistent() {
        let mut input = CalculatorInput::default();
        input.income_amount = 104_000.0;
        input.pay_frequency = PayFrequency::Weekly;

        let output = calculate_income(&input, &rules()).unwrap();
        assert_relative_eq!(output.gross_income_period, 2000.0, epsilon = 0.01);
    }

    #[test]
    fn deductions_reduce_taxable_income() {
        let mut baseline = CalculatorInput::default();
        baseline.income_amount = 100_000.0;
        let with_deductions = CalculatorInput {
            deductions_annual: 5_000.0,
            ..baseline.clone()
        };

        let a = calculate_income(&baseline, &rules()).unwrap();
        let b = calculate_income(&with_deductions, &rules()).unwrap();
        assert!(b.taxable_income_annual < a.taxable_income_annual);
        assert!(b.total_withheld_annual < a.total_withheld_annual);
    }

    #[test]
    fn bracket_breakdown_sums_to_income_tax() {
        let mut input = CalculatorInput::default();
        input.income_amount = 150_000.0;

        let output = calculate_income(&input, &rules()).unwrap();
        let sum: f64 = output.bracket_breakdown.iter().map(|b| b.tax_amount).sum();
        assert_relative_eq!(sum, output.income_tax_annual, epsilon = 0.01);
    }

    #[test]
    fn old_storage_format_still_loads() {
        let old_json = r#"{"gross_income_annual":90000.0,"pay_frequency":"Monthly","includes_super":false,"super_rate_percent":11.5,"has_help_debt":true,"deductions_annual":0.0,"salary_sacrifice_annual":0.0,"has_private_hospital_cover":false,"reportable_fringe_benefits_annual":0.0,"mls_income_for_surcharge_annual":null}"#;
        let parsed: CalculatorInput = serde_json::from_str(old_json).unwrap();
        assert_relative_eq!(parsed.income_amount, 90_000.0, epsilon = 0.01);
        assert_eq!(parsed.income_unit, IncomeUnit::Annual);
        assert_eq!(parsed.financial_year, FinancialYear::Fy2025_26);
    }
}
