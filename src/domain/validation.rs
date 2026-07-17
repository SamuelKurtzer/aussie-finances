use crate::domain::types::{CalculatorInput, IncomeUnit, ValidationIssue};

pub fn validate_input(input: &CalculatorInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if input.income_amount < 0.0 {
        issues.push(ValidationIssue {
            field: "income_amount",
            message: "Income must be zero or greater.".to_string(),
        });
    }

    if input.income_unit == IncomeUnit::Hourly && !(1.0..=100.0).contains(&input.hours_per_week) {
        issues.push(ValidationIssue {
            field: "hours_per_week",
            message: "Hours per week must be between 1 and 100.".to_string(),
        });
    }

    if input.income_unit == IncomeUnit::Daily && !(1.0..=7.0).contains(&input.days_per_week) {
        issues.push(ValidationIssue {
            field: "days_per_week",
            message: "Days per week must be between 1 and 7.".to_string(),
        });
    }

    if input.bonus_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "bonus_annual",
            message: "Bonus must be zero or greater.".to_string(),
        });
    }

    if input.overtime_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "overtime_annual",
            message: "Overtime must be zero or greater.".to_string(),
        });
    }

    if !(0.0..=25.0).contains(&input.super_rate_percent) {
        issues.push(ValidationIssue {
            field: "super_rate_percent",
            message: "Super rate must be between 0% and 25%.".to_string(),
        });
    }

    if input.deductions_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "deductions_annual",
            message: "Deductions must be zero or greater.".to_string(),
        });
    }

    if input.salary_sacrifice_amount < 0.0 {
        issues.push(ValidationIssue {
            field: "salary_sacrifice_amount",
            message: "Salary sacrifice must be zero or greater.".to_string(),
        });
    }

    if input.extra_super_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "extra_super_annual",
            message: "Extra super must be zero or greater.".to_string(),
        });
    }

    if input.reportable_fringe_benefits_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "reportable_fringe_benefits_annual",
            message: "Reportable fringe benefits must be zero or greater.".to_string(),
        });
    }

    if input.dividends_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "dividends_annual",
            message: "Dividends must be zero or greater.".to_string(),
        });
    }

    if !(0.0..=100.0).contains(&input.dividend_franking_percent) {
        issues.push(ValidationIssue {
            field: "dividend_franking_percent",
            message: "Franking percent must be between 0 and 100.".to_string(),
        });
    }

    if input.income_growth_percent < -100.0 {
        issues.push(ValidationIssue {
            field: "income_growth_percent",
            message: "Income growth must be -100% or greater.".to_string(),
        });
    }

    if input.super_balance_current < 0.0 {
        issues.push(ValidationIssue {
            field: "super_balance_current",
            message: "Super balance must be zero or greater.".to_string(),
        });
    }

    if input.super_growth_percent < 0.0 {
        issues.push(ValidationIssue {
            field: "super_growth_percent",
            message: "Super growth must be zero or greater.".to_string(),
        });
    }

    if let Some(family_income) = input.family_income_annual {
        if family_income < 0.0 {
            issues.push(ValidationIssue {
                field: "family_income_annual",
                message: "Family income must be zero or greater.".to_string(),
            });
        }
    }

    let annual_gross = input.annual_salary() + input.bonus_annual + input.overtime_annual;
    let extra_super = if input.maximize_super {
        0.0
    } else {
        input.extra_super_annual
    };
    if input.deductions_annual + input.salary_sacrifice_annualised() + extra_super > annual_gross {
        issues.push(ValidationIssue {
            field: "taxable_income_annual",
            message: "Deductions, salary sacrifice, and extra super cannot exceed gross income."
                .to_string(),
        });
    }

    issues
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::domain::types::CalculatorInput;

    fn valid_input() -> CalculatorInput {
        CalculatorInput::default()
    }

    #[test]
    fn valid_defaults_pass() {
        let issues = validate_input(&valid_input());
        assert!(issues.is_empty());
    }

    #[test]
    fn negative_income_fails() {
        let mut input = valid_input();
        input.income_amount = -1.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "income_amount"));
    }

    #[test]
    fn hourly_requires_hours_in_range() {
        let mut input = valid_input();
        input.income_unit = IncomeUnit::Hourly;
        input.hours_per_week = 0.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "hours_per_week"));
    }

    #[test]
    fn hours_ignored_for_annual_unit() {
        let mut input = valid_input();
        input.hours_per_week = 0.0;
        let issues = validate_input(&input);
        assert!(!issues.iter().any(|i| i.field == "hours_per_week"));
    }

    #[test]
    fn daily_requires_days_in_range() {
        let mut input = valid_input();
        input.income_unit = IncomeUnit::Daily;
        input.days_per_week = 8.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "days_per_week"));
    }

    #[test]
    fn negative_bonus_fails() {
        let mut input = valid_input();
        input.bonus_annual = -1.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "bonus_annual"));
    }

    #[test]
    fn negative_overtime_fails() {
        let mut input = valid_input();
        input.overtime_annual = -1.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "overtime_annual"));
    }

    #[test]
    fn negative_family_income_fails() {
        let mut input = valid_input();
        input.family_income_annual = Some(-1.0);
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "family_income_annual"));
    }

    #[test]
    fn super_rate_zero_passes() {
        let mut input = valid_input();
        input.super_rate_percent = 0.0;
        let issues = validate_input(&input);
        assert!(!issues.iter().any(|i| i.field == "super_rate_percent"));
    }

    #[test]
    fn super_rate_25_passes() {
        let mut input = valid_input();
        input.super_rate_percent = 25.0;
        let issues = validate_input(&input);
        assert!(!issues.iter().any(|i| i.field == "super_rate_percent"));
    }

    #[test]
    fn super_rate_above_25_fails() {
        let mut input = valid_input();
        input.super_rate_percent = 25.1;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "super_rate_percent"));
    }

    #[test]
    fn super_rate_negative_fails() {
        let mut input = valid_input();
        input.super_rate_percent = -0.1;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "super_rate_percent"));
    }

    #[test]
    fn negative_deductions_fails() {
        let mut input = valid_input();
        input.deductions_annual = -100.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "deductions_annual"));
    }

    #[test]
    fn negative_salary_sacrifice_fails() {
        let mut input = valid_input();
        input.salary_sacrifice_amount = -50.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "salary_sacrifice_amount"));
    }

    #[test]
    fn negative_rfb_fails() {
        let mut input = valid_input();
        input.reportable_fringe_benefits_annual = -1.0;
        let issues = validate_input(&input);
        assert!(issues
            .iter()
            .any(|i| i.field == "reportable_fringe_benefits_annual"));
    }

    #[test]
    fn deductions_plus_sacrifice_exceed_income_fails() {
        let mut input = valid_input();
        input.income_amount = 50_000.0;
        input.deductions_annual = 30_000.0;
        input.salary_sacrifice_amount = 25_000.0;
        let issues = validate_input(&input);
        assert!(issues.iter().any(|i| i.field == "taxable_income_annual"));
    }

    #[test]
    fn deductions_equal_to_income_passes() {
        let mut input = valid_input();
        input.income_amount = 50_000.0;
        input.deductions_annual = 50_000.0;
        input.salary_sacrifice_amount = 0.0;
        let issues = validate_input(&input);
        assert!(!issues.iter().any(|i| i.field == "taxable_income_annual"));
    }

    #[test]
    fn bonus_counts_toward_gross_for_deduction_check() {
        let mut input = valid_input();
        input.income_amount = 50_000.0;
        input.bonus_annual = 10_000.0;
        input.deductions_annual = 55_000.0;
        let issues = validate_input(&input);
        assert!(!issues.iter().any(|i| i.field == "taxable_income_annual"));
    }
}
