use crate::domain::calculator::income_output_for_year;
use crate::domain::mortgages::{
    AmortizationRow, DebtRecyclePeriod, MortgagePortfolioInput, MortgagePortfolioOutput,
};
use crate::domain::tax_rules::TaxRules;
use crate::domain::types::{CalculatorInput, CalculatorOutput};

/// Contributions tax inside the fund.
const SUPER_CONTRIBUTIONS_TAX: f64 = 0.15;
/// Forecast horizon when only income data exists but growth/super inputs are
/// set (matches the default 30-year mortgage horizon).
const INCOME_ONLY_FORECAST_MONTHS: usize = 360;

#[derive(Clone, PartialEq)]
pub struct SpreadsheetRow {
    pub month: usize,
    // Income columns (annual / 12 for the row's forecast year)
    pub gross_income: Option<f64>,
    pub net_income: Option<f64>,
    pub income_tax: Option<f64>,
    pub medicare: Option<f64>,
    pub help: Option<f64>,
    pub total_withheld: Option<f64>,
    // Mortgage columns
    pub opening_balance: Option<f64>,
    pub repayment: Option<f64>,
    pub interest: Option<f64>,
    pub principal: Option<f64>,
    pub closing_balance: Option<f64>,
    pub offset: Option<f64>,
    pub cumulative_interest: Option<f64>,
    // Debt recycling columns
    pub dr_draw: Option<f64>,
    pub dr_investment: Option<f64>,
    pub dr_dividend: Option<f64>,
    pub dr_franking: Option<f64>,
    pub dr_recycled_debt: Option<f64>,
    pub dr_deductible_interest: Option<f64>,
    // Wealth columns
    pub property_value: Option<f64>,
    pub super_balance: Option<f64>,
    pub net_worth: Option<f64>,
}

pub fn build_spreadsheet(
    income_input: Option<&CalculatorInput>,
    portfolio_input: Option<&MortgagePortfolioInput>,
    mortgage: Option<&MortgagePortfolioOutput>,
) -> Vec<SpreadsheetRow> {
    let monthly_mortgage = mortgage.map(build_monthly_mortgage_rows);
    let monthly_dr = mortgage
        .and_then(|m| m.debt_recycle.as_ref())
        .map(|dr| build_monthly_dr_rows(&dr.periods, mortgage.unwrap()));

    let mortgage_months = monthly_mortgage.as_ref().map(|v| v.len()).unwrap_or(0);
    let dr_months = monthly_dr.as_ref().map(|v| v.len()).unwrap_or(0);
    let income_months = match income_input {
        Some(inp)
            if inp.income_growth_percent != 0.0
                || inp.super_balance_current > 0.0
                || inp.super_growth_percent > 0.0 =>
        {
            INCOME_ONLY_FORECAST_MONTHS
        }
        Some(_) => 1,
        None => 0,
    };
    // The mortgage/DR projection sets the horizon when present; otherwise
    // income data alone drives it (one snapshot row, or a full forecast when
    // any growth/super input is set).
    let max_months = if mortgage_months > 0 || dr_months > 0 {
        mortgage_months.max(dr_months)
    } else {
        income_months
    };

    if max_months == 0 {
        return Vec::new();
    }

    // One income output per forecast year; month m uses yearly[(m-1)/12].
    let yearly: Vec<Option<CalculatorOutput>> = match income_input {
        Some(inp) => {
            let rules = TaxRules::for_year(inp.financial_year);
            (0..max_months.div_ceil(12))
                .map(|year| income_output_for_year(inp, &rules, year))
                .collect()
        }
        None => Vec::new(),
    };
    let year_output = |month: usize| -> Option<&CalculatorOutput> {
        yearly.get((month - 1) / 12).and_then(|o| o.as_ref())
    };

    // Super balance path, computed up front so rows just index into it.
    let super_balances: Option<Vec<f64>> = income_input.map(|inp| {
        let monthly_growth = (1.0 + inp.super_growth_percent / 100.0).powf(1.0 / 12.0) - 1.0;
        let mut balance = inp.super_balance_current;
        (1..=max_months)
            .map(|month| {
                let contrib_net_monthly = year_output(month)
                    .map(|y| {
                        y.concessional_contributions_annual * (1.0 - SUPER_CONTRIBUTIONS_TAX) / 12.0
                    })
                    .unwrap_or(0.0);
                balance = balance * (1.0 + monthly_growth) + contrib_net_monthly;
                balance
            })
            .collect()
    });

    let property_value_at = |month: usize| -> Option<f64> {
        portfolio_input.map(|p| {
            p.mortgages
                .iter()
                .map(|m| {
                    m.home_value
                        * (1.0 + m.property_growth_percent / 100.0).powf(month as f64 / 12.0)
                })
                .sum()
        })
    };

    (1..=max_months)
        .map(|month| {
            let mort = monthly_mortgage.as_ref().and_then(|v| v.get(month - 1));
            let dr = monthly_dr.as_ref().and_then(|v| v.get(month - 1));
            let inc = year_output(month);

            let property_value = property_value_at(month);
            let super_balance = super_balances
                .as_ref()
                .and_then(|b| b.get(month - 1))
                .copied();
            let closing_balance = mort.map(|r| r.closing_balance);
            let offset = mort.map(|r| r.offset_balance);
            let dr_investment = dr.map(|d| d.investment_value);

            let net_worth = (property_value.is_some()
                || dr_investment.is_some()
                || offset.is_some()
                || super_balance.is_some()
                || closing_balance.is_some())
            .then(|| {
                property_value.unwrap_or(0.0)
                    + dr_investment.unwrap_or(0.0)
                    + offset.unwrap_or(0.0)
                    + super_balance.unwrap_or(0.0)
                    - closing_balance.unwrap_or(0.0)
            });

            SpreadsheetRow {
                month,
                gross_income: inc.map(|y| y.gross_income_annual / 12.0),
                net_income: inc.map(|y| y.net_income_annual / 12.0),
                income_tax: inc.map(|y| y.income_tax_annual / 12.0),
                medicare: inc
                    .map(|y| (y.medicare_levy_annual + y.medicare_levy_surcharge_annual) / 12.0),
                help: inc.map(|y| y.help_repayment_annual / 12.0),
                total_withheld: inc.map(|y| y.total_withheld_annual / 12.0),
                opening_balance: mort.map(|r| r.opening_balance),
                repayment: mort.map(|r| r.repayment),
                interest: mort.map(|r| r.interest),
                principal: mort.map(|r| r.principal),
                closing_balance,
                offset,
                cumulative_interest: mort.map(|r| r.cumulative_interest),
                dr_draw: dr.map(|d| d.redraw_amount),
                dr_investment,
                dr_dividend: dr.map(|d| d.dividend_cash),
                dr_franking: dr.map(|d| d.franking_credit),
                dr_recycled_debt: dr.map(|d| d.recycled_debt_balance),
                dr_deductible_interest: dr.map(|d| d.cumulative_deductible_interest),
                property_value,
                super_balance,
                net_worth,
            }
        })
        .collect()
}

pub fn map_periods_to_months<T: Clone>(rows: &[T], period_months: &[f64]) -> Vec<T> {
    if rows.is_empty() {
        return Vec::new();
    }

    let month_series: Vec<f64> = if period_months.len() >= 2 {
        period_months.to_vec()
    } else {
        (0..=rows.len()).map(|i| i as f64).collect()
    };

    let max_month = month_series
        .last()
        .copied()
        .unwrap_or(rows.len() as f64)
        .round() as usize;

    (1..=max_month)
        .map(|month| {
            let idx = month_series
                .iter()
                .skip(1)
                .position(|m| *m >= month as f64)
                .unwrap_or(rows.len().saturating_sub(1));
            rows[idx.min(rows.len() - 1)].clone()
        })
        .collect()
}

fn build_monthly_mortgage_rows(output: &MortgagePortfolioOutput) -> Vec<AmortizationRow> {
    map_periods_to_months(
        &output.amortization_rows,
        &output.chart_series.period_months,
    )
}

fn build_monthly_dr_rows(
    periods: &[DebtRecyclePeriod],
    output: &MortgagePortfolioOutput,
) -> Vec<DebtRecyclePeriod> {
    map_periods_to_months(periods, &output.chart_series.period_months)
}

const CSV_HEADER: &str = "Month,Gross Income,Net Income,Income Tax,Medicare,HELP,Total Withheld,Opening Bal,Repayment,Interest,Principal,Closing Bal,Offset,Cum. Interest,DR Redraw,DR Investment,DR Dividend,DR Franking,DR Recycled Debt,DR Cum. Deductible Interest,Property Value,Super Balance,Net Worth";

pub fn rows_to_csv(rows: &[SpreadsheetRow]) -> String {
    let mut out = String::with_capacity(rows.len() * 200);
    out.push_str(CSV_HEADER);
    out.push('\n');

    for row in rows {
        out.push_str(&row.month.to_string());
        csv_field(&mut out, row.gross_income);
        csv_field(&mut out, row.net_income);
        csv_field(&mut out, row.income_tax);
        csv_field(&mut out, row.medicare);
        csv_field(&mut out, row.help);
        csv_field(&mut out, row.total_withheld);
        csv_field(&mut out, row.opening_balance);
        csv_field(&mut out, row.repayment);
        csv_field(&mut out, row.interest);
        csv_field(&mut out, row.principal);
        csv_field(&mut out, row.closing_balance);
        csv_field(&mut out, row.offset);
        csv_field(&mut out, row.cumulative_interest);
        csv_field(&mut out, row.dr_draw);
        csv_field(&mut out, row.dr_investment);
        csv_field(&mut out, row.dr_dividend);
        csv_field(&mut out, row.dr_franking);
        csv_field(&mut out, row.dr_recycled_debt);
        csv_field(&mut out, row.dr_deductible_interest);
        csv_field(&mut out, row.property_value);
        csv_field(&mut out, row.super_balance);
        csv_field(&mut out, row.net_worth);
        out.push('\n');
    }

    out
}

fn csv_field(out: &mut String, val: Option<f64>) {
    out.push(',');
    if let Some(v) = val {
        out.push_str(&format!("{:.2}", v));
    }
}

#[cfg(test)]
mod tests {
    use approx::assert_relative_eq;

    use super::*;
    use crate::domain::calculator::calculate_income;
    use crate::domain::mortgages::calculate_mortgage_portfolio;
    use crate::domain::types::IncomeUnit;

    #[test]
    fn map_periods_to_months_empty() {
        let result = map_periods_to_months::<i32>(&[], &[]);
        assert!(result.is_empty());
    }

    #[test]
    fn map_periods_to_months_identity_monthly() {
        let rows = vec![10, 20, 30];
        let period_months = vec![0.0, 1.0, 2.0, 3.0];
        let result = map_periods_to_months(&rows, &period_months);
        assert_eq!(result, vec![10, 20, 30]);
    }

    #[test]
    fn map_periods_to_months_non_monthly_cadence() {
        let rows = vec![100, 200];
        let period_months = vec![0.0, 0.46, 0.92];
        let result = map_periods_to_months(&rows, &period_months);
        assert_eq!(result.len(), 1);
    }

    #[test]
    fn build_spreadsheet_empty() {
        let rows = build_spreadsheet(None, None, None);
        assert!(rows.is_empty());
    }

    #[test]
    fn build_spreadsheet_income_only_snapshot_is_one_row() {
        let income = CalculatorInput::default();
        let rows = build_spreadsheet(Some(&income), None, None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].month, 1);
        assert!(rows[0].gross_income.is_some());
        assert!(rows[0].opening_balance.is_none());
    }

    #[test]
    fn income_only_forecast_inputs_extend_horizon() {
        let income = CalculatorInput {
            super_balance_current: 50_000.0,
            super_growth_percent: 7.0,
            ..Default::default()
        };
        let rows = build_spreadsheet(Some(&income), None, None);
        assert_eq!(rows.len(), 360);
    }

    #[test]
    fn build_spreadsheet_mortgage_only() {
        let input = MortgagePortfolioInput::default();
        let output = calculate_mortgage_portfolio(&input, None).unwrap();
        let rows = build_spreadsheet(None, Some(&input), Some(&output));
        assert!(!rows.is_empty());
        assert!(rows[0].gross_income.is_none());
        assert!(rows[0].opening_balance.is_some());
    }

    #[test]
    fn net_worth_is_assets_minus_debt_each_month() {
        let income = CalculatorInput {
            super_balance_current: 100_000.0,
            ..Default::default()
        };
        let portfolio = MortgagePortfolioInput::default();
        let output = calculate_mortgage_portfolio(&portfolio, None).unwrap();

        let rows = build_spreadsheet(Some(&income), Some(&portfolio), Some(&output));
        for row in [&rows[0], rows.last().unwrap()] {
            let expected = row.property_value.unwrap()
                + row.dr_investment.unwrap_or(0.0)
                + row.offset.unwrap()
                + row.super_balance.unwrap()
                - row.closing_balance.unwrap();
            assert_relative_eq!(row.net_worth.unwrap(), expected, epsilon = 0.01);
        }
        // The loan is paid off by the final row, so net worth is pure assets.
        let last = rows.last().unwrap();
        assert!(last.closing_balance.unwrap() < 1.0);
        assert!(last.net_worth.unwrap() > rows[0].net_worth.unwrap());
    }

    #[test]
    fn income_growth_recomputes_tax_per_year() {
        let income = CalculatorInput {
            income_growth_percent: 3.0,
            super_balance_current: 1.0, // activate the forecast horizon
            ..Default::default()
        };

        let rows = build_spreadsheet(Some(&income), None, None);
        let rules = TaxRules::for_year(income.financial_year);

        // Month 13 must match a hand-scaled full recompute exactly.
        let mut year2 = income.clone();
        year2.income_amount = income.annual_salary() * 1.03;
        year2.income_unit = IncomeUnit::Annual;
        let expected = calculate_income(&year2, &rules).unwrap();
        assert_relative_eq!(
            rows[12].net_income.unwrap(),
            expected.net_income_annual / 12.0,
            epsilon = 0.01
        );

        // Bracket creep: year 2 loses a larger share to tax than year 1.
        let rate = |row: &SpreadsheetRow| row.total_withheld.unwrap() / row.gross_income.unwrap();
        assert!(rate(&rows[12]) > rate(&rows[0]));
    }

    #[test]
    fn hourly_income_grows_from_annualised_base() {
        let income = CalculatorInput {
            income_amount: 50.0,
            income_unit: IncomeUnit::Hourly,
            hours_per_week: 38.0,
            income_growth_percent: 3.0,
            ..Default::default()
        };

        let rows = build_spreadsheet(Some(&income), None, None);
        let base_annual = 50.0 * 38.0 * 52.0;
        assert_relative_eq!(
            rows[12].gross_income.unwrap(),
            base_annual * 1.03 / 12.0,
            epsilon = 0.01
        );
    }

    #[test]
    fn super_balance_compounds_with_net_contributions() {
        let income = CalculatorInput {
            super_balance_current: 10_000.0,
            ..Default::default()
        };
        // growth 0: balance is start plus net-of-tax contributions.
        let rows = build_spreadsheet(Some(&income), None, None);
        let rules = TaxRules::for_year(income.financial_year);
        let contrib = calculate_income(&income, &rules)
            .unwrap()
            .concessional_contributions_annual;
        assert_relative_eq!(
            rows[23].super_balance.unwrap(),
            10_000.0 + 2.0 * contrib * 0.85,
            epsilon = 0.01
        );

        let mut growing = income.clone();
        growing.super_growth_percent = 7.0;
        let growing_rows = build_spreadsheet(Some(&growing), None, None);
        assert!(growing_rows[23].super_balance.unwrap() > rows[23].super_balance.unwrap());
    }

    #[test]
    fn super_column_absent_without_income() {
        let portfolio = MortgagePortfolioInput::default();
        let output = calculate_mortgage_portfolio(&portfolio, None).unwrap();
        let rows = build_spreadsheet(None, Some(&portfolio), Some(&output));
        assert!(rows[0].super_balance.is_none());
    }

    #[test]
    fn property_value_grows_at_configured_rate() {
        let mut portfolio = MortgagePortfolioInput::default();
        let output = calculate_mortgage_portfolio(&portfolio, None).unwrap();

        let flat = build_spreadsheet(None, Some(&portfolio), Some(&output));
        assert_relative_eq!(flat[11].property_value.unwrap(), 750_000.0, epsilon = 0.01);

        portfolio.mortgages[0].property_growth_percent = 5.0;
        let grown = build_spreadsheet(None, Some(&portfolio), Some(&output));
        assert_relative_eq!(
            grown[11].property_value.unwrap(),
            750_000.0 * 1.05,
            epsilon = 1.0
        );
    }

    #[test]
    fn old_mortgage_json_without_property_growth_still_loads() {
        let raw = r#"{
            "id": 1,
            "name": "Mortgage 1",
            "home_value": 750000.0,
            "offset_balance": 20000.0,
            "term_months": 360,
            "splits": []
        }"#;
        let parsed = serde_json::from_str::<crate::domain::mortgages::MortgageInput>(raw).unwrap();
        assert_relative_eq!(parsed.property_growth_percent, 0.0, epsilon = 0.001);
    }

    #[test]
    fn rows_to_csv_header_present() {
        let rows = vec![SpreadsheetRow {
            month: 1,
            gross_income: Some(10_000.0),
            net_income: Some(8_000.0),
            income_tax: Some(2_000.0),
            medicare: Some(200.0),
            help: Some(0.0),
            total_withheld: Some(2_200.0),
            opening_balance: None,
            repayment: None,
            interest: None,
            principal: None,
            closing_balance: None,
            offset: None,
            cumulative_interest: None,
            dr_draw: None,
            dr_investment: None,
            dr_dividend: None,
            dr_franking: None,
            dr_recycled_debt: None,
            dr_deductible_interest: None,
            property_value: None,
            super_balance: None,
            net_worth: None,
        }];
        let csv = rows_to_csv(&rows);
        assert!(csv.starts_with("Month,Gross Income,"));
        let lines: Vec<&str> = csv.trim().lines().collect();
        assert_eq!(lines.len(), 2);
    }

    #[test]
    fn rows_to_csv_empty_fields_render_blank() {
        let rows = vec![SpreadsheetRow {
            month: 1,
            gross_income: None,
            net_income: None,
            income_tax: None,
            medicare: None,
            help: None,
            total_withheld: None,
            opening_balance: Some(500_000.0),
            repayment: Some(3_000.0),
            interest: Some(2_500.0),
            principal: Some(500.0),
            closing_balance: Some(499_500.0),
            offset: Some(20_000.0),
            cumulative_interest: Some(2_500.0),
            dr_draw: None,
            dr_investment: None,
            dr_dividend: None,
            dr_franking: None,
            dr_recycled_debt: None,
            dr_deductible_interest: None,
            property_value: None,
            super_balance: None,
            net_worth: None,
        }];
        let csv = rows_to_csv(&rows);
        let data_line = csv.lines().nth(1).unwrap();
        let fields: Vec<&str> = data_line.split(',').collect();
        assert_eq!(fields[0], "1");
        assert_eq!(fields[1], "");
        assert_eq!(fields[7], "500000.00");
    }
}
