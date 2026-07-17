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

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum FinancialYear {
    Fy2024_25,
    #[default]
    Fy2025_26,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum IncomeUnit {
    Hourly,
    Daily,
    Weekly,
    Fortnightly,
    Monthly,
    #[default]
    Annual,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum ContributionFrequency {
    #[default]
    Annual,
    Monthly,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum MedicareExemption {
    #[default]
    None,
    Half,
    Full,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Residency {
    #[default]
    Resident,
    NonResident,
    WorkingHolidayMaker,
}

impl Residency {
    pub fn as_str(self) -> &'static str {
        match self {
            Self::Resident => "Australian Resident",
            Self::NonResident => "Non-resident",
            Self::WorkingHolidayMaker => "Working Holiday Maker",
        }
    }
}

#[derive(Clone, Debug, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct CalculatorInput {
    #[serde(alias = "gross_income_annual")]
    pub income_amount: f64,
    pub income_unit: IncomeUnit,
    pub hours_per_week: f64,
    pub days_per_week: f64,
    pub financial_year: FinancialYear,
    pub pay_frequency: PayFrequency,
    pub residency: Residency,
    pub bonus_annual: f64,
    pub overtime_annual: f64,
    pub includes_super: bool,
    pub super_rate_percent: f64,
    pub extra_super_annual: f64,
    pub maximize_super: bool,
    pub has_help_debt: bool,
    pub deductions_annual: f64,
    #[serde(alias = "salary_sacrifice_annual")]
    pub salary_sacrifice_amount: f64,
    pub salary_sacrifice_frequency: ContributionFrequency,
    pub medicare_exemption: MedicareExemption,
    pub is_sapto_eligible: bool,
    pub has_family: bool,
    pub dependants: u32,
    pub family_income_annual: Option<f64>,
    pub has_private_hospital_cover: bool,
    pub reportable_fringe_benefits_annual: f64,
    pub mls_income_for_surcharge_annual: Option<f64>,
    pub dividends_annual: f64,
    pub dividend_franking_percent: f64,
    /// Company tax rate used for the franking gross-up (30% for most listed
    /// companies; base-rate entities frank at 25%).
    pub dividend_company_tax_rate_percent: f64,
    /// Fill dividends/franking from the Debt Recycling projection's first
    /// year instead of the manual fields above.
    pub link_dividends_to_dr: bool,
    /// Annual salary growth applied per forecast year in the spreadsheet.
    pub income_growth_percent: f64,
    pub super_balance_current: f64,
    pub super_growth_percent: f64,
}

impl CalculatorInput {
    pub fn annual_salary(&self) -> f64 {
        match self.income_unit {
            IncomeUnit::Hourly => self.income_amount * self.hours_per_week * 52.0,
            IncomeUnit::Daily => self.income_amount * self.days_per_week * 52.0,
            IncomeUnit::Weekly => self.income_amount * 52.0,
            IncomeUnit::Fortnightly => self.income_amount * 26.0,
            IncomeUnit::Monthly => self.income_amount * 12.0,
            IncomeUnit::Annual => self.income_amount,
        }
    }

    pub fn salary_sacrifice_annualised(&self) -> f64 {
        match self.salary_sacrifice_frequency {
            ContributionFrequency::Annual => self.salary_sacrifice_amount,
            ContributionFrequency::Monthly => self.salary_sacrifice_amount * 12.0,
        }
    }
}

impl Default for CalculatorInput {
    fn default() -> Self {
        Self {
            income_amount: 100_000.0,
            income_unit: IncomeUnit::Annual,
            hours_per_week: 38.0,
            days_per_week: 5.0,
            financial_year: FinancialYear::Fy2025_26,
            pay_frequency: PayFrequency::Fortnightly,
            residency: Residency::Resident,
            bonus_annual: 0.0,
            overtime_annual: 0.0,
            includes_super: false,
            super_rate_percent: 12.0,
            extra_super_annual: 0.0,
            maximize_super: false,
            has_help_debt: false,
            deductions_annual: 0.0,
            salary_sacrifice_amount: 0.0,
            salary_sacrifice_frequency: ContributionFrequency::Annual,
            medicare_exemption: MedicareExemption::None,
            is_sapto_eligible: false,
            has_family: false,
            dependants: 0,
            family_income_annual: None,
            has_private_hospital_cover: false,
            reportable_fringe_benefits_annual: 0.0,
            mls_income_for_surcharge_annual: None,
            dividends_annual: 0.0,
            dividend_franking_percent: 100.0,
            dividend_company_tax_rate_percent: 30.0,
            link_dividends_to_dr: false,
            income_growth_percent: 0.0,
            super_balance_current: 0.0,
            super_growth_percent: 0.0,
        }
    }
}

#[derive(Clone, Debug, PartialEq)]
pub struct BracketLine {
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub rate: f64,
    pub tax_amount: f64,
}

#[derive(Clone, Debug, PartialEq)]
pub struct CalculatorOutput {
    pub gross_income_annual: f64,
    pub gross_income_period: f64,
    pub taxable_income_annual: f64,
    pub income_tax_annual: f64,
    pub lito_annual: f64,
    pub sapto_annual: f64,
    pub medicare_levy_annual: f64,
    pub medicare_levy_surcharge_annual: f64,
    pub help_repayment_annual: f64,
    pub total_withheld_annual: f64,
    pub net_income_annual: f64,
    pub net_income_period: f64,
    pub effective_tax_rate_percent: f64,
    pub marginal_rate_percent: f64,
    pub super_guarantee_annual: f64,
    pub concessional_contributions_annual: f64,
    pub division_293_annual: f64,
    pub dividends_annual: f64,
    pub franking_credits_annual: f64,
    pub bracket_breakdown: Vec<BracketLine>,
    pub pay_frequency: PayFrequency,
    pub warnings: Vec<String>,
    pub assumptions: Vec<String>,
}

#[derive(Clone, Debug, PartialEq)]
pub enum DomainError {
    Validation(Vec<ValidationIssue>),
}

pub type CalculatorError = DomainError;

#[derive(Clone, Debug, PartialEq)]
pub struct ValidationIssue {
    pub field: &'static str,
    pub message: String,
}
