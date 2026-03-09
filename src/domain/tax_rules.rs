#[derive(Clone, Debug)]
pub struct TaxBracket {
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub rate: f64,
}

#[derive(Clone, Debug)]
pub struct HelpBracket {
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub rate: f64,
}

#[derive(Clone, Debug)]
pub struct MlsTier {
    pub lower_bound: f64,
    pub upper_bound: Option<f64>,
    pub rate: f64,
}

#[derive(Clone, Debug)]
pub struct TaxRules {
    pub name: &'static str,
    pub resident_tax_brackets: Vec<TaxBracket>,
    pub medicare_levy_rate: f64,
    pub medicare_levy_low_income_threshold: f64,
    pub help_brackets: Vec<HelpBracket>,
    pub mls_tiers: Vec<MlsTier>,
}

impl TaxRules {
    pub fn fy_2025_26_resident() -> Self {
        Self {
            name: "FY 2025-26 Resident (MVP)",
            resident_tax_brackets: vec![
                TaxBracket {
                    lower_bound: 0.0,
                    upper_bound: Some(18_200.0),
                    rate: 0.0,
                },
                TaxBracket {
                    lower_bound: 18_200.0,
                    upper_bound: Some(45_000.0),
                    rate: 0.16,
                },
                TaxBracket {
                    lower_bound: 45_000.0,
                    upper_bound: Some(135_000.0),
                    rate: 0.30,
                },
                TaxBracket {
                    lower_bound: 135_000.0,
                    upper_bound: Some(190_000.0),
                    rate: 0.37,
                },
                TaxBracket {
                    lower_bound: 190_000.0,
                    upper_bound: None,
                    rate: 0.45,
                },
            ],
            medicare_levy_rate: 0.02,
            medicare_levy_low_income_threshold: 26_000.0,
            help_brackets: vec![
                HelpBracket {
                    lower_bound: 54_435.0,
                    upper_bound: Some(62_850.0),
                    rate: 0.01,
                },
                HelpBracket {
                    lower_bound: 62_850.0,
                    upper_bound: Some(66_620.0),
                    rate: 0.02,
                },
                HelpBracket {
                    lower_bound: 66_620.0,
                    upper_bound: Some(70_618.0),
                    rate: 0.025,
                },
                HelpBracket {
                    lower_bound: 70_618.0,
                    upper_bound: Some(74_855.0),
                    rate: 0.03,
                },
                HelpBracket {
                    lower_bound: 74_855.0,
                    upper_bound: Some(79_346.0),
                    rate: 0.035,
                },
                HelpBracket {
                    lower_bound: 79_346.0,
                    upper_bound: Some(84_106.0),
                    rate: 0.04,
                },
                HelpBracket {
                    lower_bound: 84_106.0,
                    upper_bound: Some(89_153.0),
                    rate: 0.045,
                },
                HelpBracket {
                    lower_bound: 89_153.0,
                    upper_bound: Some(94_502.0),
                    rate: 0.05,
                },
                HelpBracket {
                    lower_bound: 94_502.0,
                    upper_bound: Some(100_172.0),
                    rate: 0.055,
                },
                HelpBracket {
                    lower_bound: 100_172.0,
                    upper_bound: Some(106_182.0),
                    rate: 0.06,
                },
                HelpBracket {
                    lower_bound: 106_182.0,
                    upper_bound: Some(112_553.0),
                    rate: 0.065,
                },
                HelpBracket {
                    lower_bound: 112_553.0,
                    upper_bound: Some(119_306.0),
                    rate: 0.07,
                },
                HelpBracket {
                    lower_bound: 119_306.0,
                    upper_bound: Some(126_464.0),
                    rate: 0.075,
                },
                HelpBracket {
                    lower_bound: 126_464.0,
                    upper_bound: Some(134_052.0),
                    rate: 0.08,
                },
                HelpBracket {
                    lower_bound: 134_052.0,
                    upper_bound: Some(142_095.0),
                    rate: 0.085,
                },
                HelpBracket {
                    lower_bound: 142_095.0,
                    upper_bound: Some(150_621.0),
                    rate: 0.09,
                },
                HelpBracket {
                    lower_bound: 150_621.0,
                    upper_bound: Some(159_658.0),
                    rate: 0.095,
                },
                HelpBracket {
                    lower_bound: 159_658.0,
                    upper_bound: None,
                    rate: 0.10,
                },
            ],
            mls_tiers: vec![
                MlsTier {
                    lower_bound: 97_000.0,
                    upper_bound: Some(113_000.0),
                    rate: 0.01,
                },
                MlsTier {
                    lower_bound: 113_000.0,
                    upper_bound: Some(151_000.0),
                    rate: 0.0125,
                },
                MlsTier {
                    lower_bound: 151_000.0,
                    upper_bound: None,
                    rate: 0.015,
                },
            ],
        }
    }
}
