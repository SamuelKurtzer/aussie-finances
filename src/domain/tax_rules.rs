use crate::domain::types::FinancialYear;

#[derive(Clone, Debug)]
pub struct Bracket {
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub rate: f64,
}

#[derive(Clone, Copy, Debug, PartialEq, Eq)]
pub enum HelpSystem {
    /// Pre-2025-26: a single average rate applied to the whole repayment income.
    AverageRate,
    /// From 2025-26: marginal rates above a threshold, capped at a share of income.
    Marginal,
}

#[derive(Clone, Debug)]
pub struct TaxRules {
    pub name: &'static str,
    pub resident_tax_brackets: Vec<Bracket>,
    pub non_resident_tax_brackets: Vec<Bracket>,
    pub whm_tax_brackets: Vec<Bracket>,
    pub medicare_levy_rate: f64,
    pub medicare_levy_low_income_threshold: f64,
    pub medicare_levy_family_threshold: f64,
    pub medicare_levy_child_increment: f64,
    pub medicare_levy_sapto_threshold: f64,
    pub medicare_levy_sapto_family_threshold: f64,
    pub medicare_levy_phase_in_rate: f64,
    pub help_system: HelpSystem,
    pub help_brackets: Vec<Bracket>,
    pub help_repayment_cap_rate: f64,
    pub mls_tiers: Vec<Bracket>,
    pub mls_family_tiers: Vec<Bracket>,
    pub mls_family_child_increment: f64,
    pub lito_max_offset: f64,
    pub lito_taper_one_start: f64,
    pub lito_taper_one_rate: f64,
    pub lito_taper_two_start: f64,
    pub lito_taper_two_rate: f64,
    pub sapto_max_offset: f64,
    pub sapto_taper_start: f64,
    pub sapto_taper_rate: f64,
    pub concessional_contributions_cap: f64,
    pub division_293_threshold: f64,
    pub division_293_rate: f64,
}

fn stage_three_resident_brackets() -> Vec<Bracket> {
    vec![
        Bracket {
            lower_bound: 0.0,
            upper_bound: Some(18_200.0),
            rate: 0.0,
        },
        Bracket {
            lower_bound: 18_200.0,
            upper_bound: Some(45_000.0),
            rate: 0.16,
        },
        Bracket {
            lower_bound: 45_000.0,
            upper_bound: Some(135_000.0),
            rate: 0.30,
        },
        Bracket {
            lower_bound: 135_000.0,
            upper_bound: Some(190_000.0),
            rate: 0.37,
        },
        Bracket {
            lower_bound: 190_000.0,
            upper_bound: None,
            rate: 0.45,
        },
    ]
}

fn non_resident_brackets() -> Vec<Bracket> {
    vec![
        Bracket {
            lower_bound: 0.0,
            upper_bound: Some(135_000.0),
            rate: 0.30,
        },
        Bracket {
            lower_bound: 135_000.0,
            upper_bound: Some(190_000.0),
            rate: 0.37,
        },
        Bracket {
            lower_bound: 190_000.0,
            upper_bound: None,
            rate: 0.45,
        },
    ]
}

fn whm_brackets() -> Vec<Bracket> {
    vec![
        Bracket {
            lower_bound: 0.0,
            upper_bound: Some(45_000.0),
            rate: 0.15,
        },
        Bracket {
            lower_bound: 45_000.0,
            upper_bound: Some(135_000.0),
            rate: 0.30,
        },
        Bracket {
            lower_bound: 135_000.0,
            upper_bound: Some(190_000.0),
            rate: 0.37,
        },
        Bracket {
            lower_bound: 190_000.0,
            upper_bound: None,
            rate: 0.45,
        },
    ]
}

fn mls_tiers(t1: f64, t2: f64, t3: f64) -> Vec<Bracket> {
    vec![
        Bracket {
            lower_bound: t1,
            upper_bound: Some(t2),
            rate: 0.01,
        },
        Bracket {
            lower_bound: t2,
            upper_bound: Some(t3),
            rate: 0.0125,
        },
        Bracket {
            lower_bound: t3,
            upper_bound: None,
            rate: 0.015,
        },
    ]
}

impl TaxRules {
    pub fn for_year(year: FinancialYear) -> Self {
        match year {
            FinancialYear::Fy2024_25 => Self::fy_2024_25(),
            FinancialYear::Fy2025_26 => Self::fy_2025_26(),
        }
    }

    pub fn fy_2025_26() -> Self {
        Self {
            name: "FY 2025-26",
            resident_tax_brackets: stage_three_resident_brackets(),
            non_resident_tax_brackets: non_resident_brackets(),
            whm_tax_brackets: whm_brackets(),
            medicare_levy_rate: 0.02,
            medicare_levy_low_income_threshold: 28_011.0,
            medicare_levy_family_threshold: 47_238.0,
            medicare_levy_child_increment: 4_338.0,
            medicare_levy_sapto_threshold: 44_267.0,
            medicare_levy_sapto_family_threshold: 61_622.0,
            medicare_levy_phase_in_rate: 0.10,
            // FY 2025-26 marginal HELP repayment system: nil below $67k,
            // 15% of income between $67k-$125k, 17% above $125k,
            // capped at 10% of total repayment income.
            help_system: HelpSystem::Marginal,
            help_brackets: vec![
                Bracket {
                    lower_bound: 67_000.0,
                    upper_bound: Some(125_000.0),
                    rate: 0.15,
                },
                Bracket {
                    lower_bound: 125_000.0,
                    upper_bound: None,
                    rate: 0.17,
                },
            ],
            help_repayment_cap_rate: 0.10,
            mls_tiers: mls_tiers(101_000.0, 118_000.0, 158_000.0),
            mls_family_tiers: mls_tiers(202_000.0, 236_000.0, 316_000.0),
            mls_family_child_increment: 1_500.0,
            lito_max_offset: 700.0,
            lito_taper_one_start: 37_500.0,
            lito_taper_one_rate: 0.05,
            lito_taper_two_start: 45_000.0,
            lito_taper_two_rate: 0.015,
            sapto_max_offset: 2_230.0,
            sapto_taper_start: 32_279.0,
            sapto_taper_rate: 0.125,
            concessional_contributions_cap: 30_000.0,
            division_293_threshold: 250_000.0,
            division_293_rate: 0.15,
        }
    }

    pub fn fy_2024_25() -> Self {
        Self {
            name: "FY 2024-25",
            resident_tax_brackets: stage_three_resident_brackets(),
            non_resident_tax_brackets: non_resident_brackets(),
            whm_tax_brackets: whm_brackets(),
            medicare_levy_rate: 0.02,
            medicare_levy_low_income_threshold: 27_222.0,
            medicare_levy_family_threshold: 45_907.0,
            medicare_levy_child_increment: 4_216.0,
            medicare_levy_sapto_threshold: 43_020.0,
            medicare_levy_sapto_family_threshold: 59_886.0,
            medicare_levy_phase_in_rate: 0.10,
            // Old-style HELP: average rate applied to the whole repayment income.
            help_system: HelpSystem::AverageRate,
            help_brackets: vec![
                Bracket {
                    lower_bound: 54_435.0,
                    upper_bound: Some(62_850.0),
                    rate: 0.01,
                },
                Bracket {
                    lower_bound: 62_850.0,
                    upper_bound: Some(66_620.0),
                    rate: 0.02,
                },
                Bracket {
                    lower_bound: 66_620.0,
                    upper_bound: Some(70_618.0),
                    rate: 0.025,
                },
                Bracket {
                    lower_bound: 70_618.0,
                    upper_bound: Some(74_855.0),
                    rate: 0.03,
                },
                Bracket {
                    lower_bound: 74_855.0,
                    upper_bound: Some(79_346.0),
                    rate: 0.035,
                },
                Bracket {
                    lower_bound: 79_346.0,
                    upper_bound: Some(84_106.0),
                    rate: 0.04,
                },
                Bracket {
                    lower_bound: 84_106.0,
                    upper_bound: Some(89_153.0),
                    rate: 0.045,
                },
                Bracket {
                    lower_bound: 89_153.0,
                    upper_bound: Some(94_502.0),
                    rate: 0.05,
                },
                Bracket {
                    lower_bound: 94_502.0,
                    upper_bound: Some(100_172.0),
                    rate: 0.055,
                },
                Bracket {
                    lower_bound: 100_172.0,
                    upper_bound: Some(106_182.0),
                    rate: 0.06,
                },
                Bracket {
                    lower_bound: 106_182.0,
                    upper_bound: Some(112_553.0),
                    rate: 0.065,
                },
                Bracket {
                    lower_bound: 112_553.0,
                    upper_bound: Some(119_306.0),
                    rate: 0.07,
                },
                Bracket {
                    lower_bound: 119_306.0,
                    upper_bound: Some(126_464.0),
                    rate: 0.075,
                },
                Bracket {
                    lower_bound: 126_464.0,
                    upper_bound: Some(134_052.0),
                    rate: 0.08,
                },
                Bracket {
                    lower_bound: 134_052.0,
                    upper_bound: Some(142_095.0),
                    rate: 0.085,
                },
                Bracket {
                    lower_bound: 142_095.0,
                    upper_bound: Some(150_621.0),
                    rate: 0.09,
                },
                Bracket {
                    lower_bound: 150_621.0,
                    upper_bound: Some(159_658.0),
                    rate: 0.095,
                },
                Bracket {
                    lower_bound: 159_658.0,
                    upper_bound: None,
                    rate: 0.10,
                },
            ],
            help_repayment_cap_rate: 0.10,
            mls_tiers: mls_tiers(97_000.0, 113_000.0, 151_000.0),
            mls_family_tiers: mls_tiers(194_000.0, 226_000.0, 302_000.0),
            mls_family_child_increment: 1_500.0,
            lito_max_offset: 700.0,
            lito_taper_one_start: 37_500.0,
            lito_taper_one_rate: 0.05,
            lito_taper_two_start: 45_000.0,
            lito_taper_two_rate: 0.015,
            sapto_max_offset: 2_230.0,
            sapto_taper_start: 32_279.0,
            sapto_taper_rate: 0.125,
            concessional_contributions_cap: 30_000.0,
            division_293_threshold: 250_000.0,
            division_293_rate: 0.15,
        }
    }
}
