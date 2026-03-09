use core::fmt;
use serde::{Deserialize, Serialize};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize)]
pub enum PayFrequency {
    Weekly,
    Fortnightly,
    Monthly,
    Annually,
}

impl PayFrequency {
    pub fn periods_per_year(self) -> f64 {
        match self {
            Self::Weekly => 52.0,
            Self::Fortnightly => 26.0,
            Self::Monthly => 12.0,
            Self::Annually => 1.0,
        }
    }

    pub fn as_str(self) -> &'static str {
        match self {
            Self::Weekly => "Weekly",
            Self::Fortnightly => "Fortnightly",
            Self::Monthly => "Monthly",
            Self::Annually => "Annually",
        }
    }
}

impl fmt::Display for PayFrequency {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.as_str())
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
pub struct CalculatorInput {
    pub gross_income_annual: f64,
    pub pay_frequency: PayFrequency,
    pub includes_super: bool,
    pub super_rate_percent: f64,
    pub has_help_debt: bool,
    pub deductions_annual: f64,
    pub salary_sacrifice_annual: f64,
    pub has_private_hospital_cover: bool,
    pub reportable_fringe_benefits_annual: f64,
    pub mls_income_for_surcharge_annual: Option<f64>,
}

impl Default for CalculatorInput {
    fn default() -> Self {
        Self {
            gross_income_annual: 100_000.0,
            pay_frequency: PayFrequency::Fortnightly,
            includes_super: false,
            super_rate_percent: 11.5,
            has_help_debt: false,
            deductions_annual: 0.0,
            salary_sacrifice_annual: 0.0,
            has_private_hospital_cover: false,
            reportable_fringe_benefits_annual: 0.0,
            mls_income_for_surcharge_annual: None,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalculatorOutput {
    pub gross_income_annual: f64,
    pub gross_income_period: f64,
    pub taxable_income_annual: f64,
    pub income_tax_annual: f64,
    pub medicare_levy_annual: f64,
    pub medicare_levy_surcharge_annual: f64,
    pub help_repayment_annual: f64,
    pub total_withheld_annual: f64,
    pub net_income_annual: f64,
    pub net_income_period: f64,
    pub effective_tax_rate_percent: f64,
    pub pay_frequency: PayFrequency,
    pub assumptions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum CalculatorError {
    Validation(Vec<ValidationIssue>),
}

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationIssue {
    pub field: &'static str,
    pub message: String,
}
