use crate::domain::types::{CalculatorInput, ValidationIssue};

pub fn validate_input(input: &CalculatorInput) -> Vec<ValidationIssue> {
    let mut issues = Vec::new();

    if input.gross_income_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "gross_income_annual",
            message: "Gross income must be zero or greater.".to_string(),
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

    if input.salary_sacrifice_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "salary_sacrifice_annual",
            message: "Salary sacrifice must be zero or greater.".to_string(),
        });
    }

    if input.reportable_fringe_benefits_annual < 0.0 {
        issues.push(ValidationIssue {
            field: "reportable_fringe_benefits_annual",
            message: "Reportable fringe benefits must be zero or greater.".to_string(),
        });
    }

    if input.deductions_annual + input.salary_sacrifice_annual > input.gross_income_annual {
        issues.push(ValidationIssue {
            field: "taxable_income_annual",
            message: "Deductions plus salary sacrifice cannot exceed gross income.".to_string(),
        });
    }

    issues
}
