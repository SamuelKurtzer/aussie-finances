use crate::domain::mortgages::{AmortizationRow, DebtRecyclePeriod, MortgagePortfolioOutput};
use crate::domain::types::CalculatorOutput;

#[derive(Clone, PartialEq)]
pub struct SpreadsheetRow {
    pub month: usize,
    // Income columns (annual / 12, static per row)
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
}

pub fn build_spreadsheet(
    income: Option<&CalculatorOutput>,
    mortgage: Option<&MortgagePortfolioOutput>,
) -> Vec<SpreadsheetRow> {
    let monthly_mortgage = mortgage.map(build_monthly_mortgage_rows);
    let monthly_dr = mortgage
        .and_then(|m| m.debt_recycle.as_ref())
        .map(|dr| build_monthly_dr_rows(&dr.periods, mortgage.unwrap()));

    let mortgage_months = monthly_mortgage.as_ref().map(|v| v.len()).unwrap_or(0);
    let dr_months = monthly_dr.as_ref().map(|v| v.len()).unwrap_or(0);
    let max_months = mortgage_months
        .max(dr_months)
        .max(if income.is_some() { 1 } else { 0 });

    if max_months == 0 {
        return Vec::new();
    }

    let (gross_monthly, net_monthly, tax_monthly, medicare_monthly, help_monthly, withheld_monthly) =
        match income {
            Some(inc) => (
                Some(inc.gross_income_annual / 12.0),
                Some(inc.net_income_annual / 12.0),
                Some(inc.income_tax_annual / 12.0),
                Some((inc.medicare_levy_annual + inc.medicare_levy_surcharge_annual) / 12.0),
                Some(inc.help_repayment_annual / 12.0),
                Some(inc.total_withheld_annual / 12.0),
            ),
            None => (None, None, None, None, None, None),
        };

    (1..=max_months)
        .map(|month| {
            let mort = monthly_mortgage.as_ref().and_then(|v| v.get(month - 1));
            let dr = monthly_dr.as_ref().and_then(|v| v.get(month - 1));

            SpreadsheetRow {
                month,
                gross_income: gross_monthly,
                net_income: net_monthly,
                income_tax: tax_monthly,
                medicare: medicare_monthly,
                help: help_monthly,
                total_withheld: withheld_monthly,
                opening_balance: mort.map(|r| r.opening_balance),
                repayment: mort.map(|r| r.repayment),
                interest: mort.map(|r| r.interest),
                principal: mort.map(|r| r.principal),
                closing_balance: mort.map(|r| r.closing_balance),
                offset: mort.map(|r| r.offset_balance),
                cumulative_interest: mort.map(|r| r.cumulative_interest),
                dr_draw: dr.map(|d| d.redraw_amount),
                dr_investment: dr.map(|d| d.investment_value),
                dr_dividend: dr.map(|d| d.dividend_cash),
                dr_franking: dr.map(|d| d.franking_credit),
                dr_recycled_debt: dr.map(|d| d.recycled_debt_balance),
                dr_deductible_interest: dr.map(|d| d.cumulative_deductible_interest),
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

const CSV_HEADER: &str = "Month,Gross Income,Net Income,Income Tax,Medicare,HELP,Total Withheld,Opening Bal,Repayment,Interest,Principal,Closing Bal,Offset,Cum. Interest,DR Redraw,DR Investment,DR Dividend,DR Franking,DR Recycled Debt,DR Cum. Deductible Interest";

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
    use super::*;
    use crate::domain::mortgages::{calculate_mortgage_portfolio, MortgagePortfolioInput};
    use crate::domain::types::{CalculatorOutput, PayFrequency};

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
        let rows = build_spreadsheet(None, None);
        assert!(rows.is_empty());
    }

    #[test]
    fn build_spreadsheet_income_only() {
        let income = CalculatorOutput {
            gross_income_annual: 120_000.0,
            gross_income_period: 120_000.0 / 12.0,
            taxable_income_annual: 120_000.0,
            income_tax_annual: 24_000.0,
            medicare_levy_annual: 2_400.0,
            medicare_levy_surcharge_annual: 0.0,
            help_repayment_annual: 0.0,
            total_withheld_annual: 26_400.0,
            net_income_annual: 93_600.0,
            net_income_period: 93_600.0 / 12.0,
            effective_tax_rate_percent: 22.0,
            marginal_rate_percent: 30.0,
            lito_annual: 0.0,
            sapto_annual: 0.0,
            super_guarantee_annual: 14_400.0,
            concessional_contributions_annual: 14_400.0,
            division_293_annual: 0.0,
            bracket_breakdown: vec![],
            pay_frequency: PayFrequency::Monthly,
            warnings: vec![],
            assumptions: vec![],
        };
        let rows = build_spreadsheet(Some(&income), None);
        assert_eq!(rows.len(), 1);
        assert_eq!(rows[0].month, 1);
        assert!(rows[0].gross_income.is_some());
        assert!(rows[0].opening_balance.is_none());
    }

    #[test]
    fn build_spreadsheet_mortgage_only() {
        let input = MortgagePortfolioInput::default();
        let output = calculate_mortgage_portfolio(&input, None).unwrap();
        let rows = build_spreadsheet(None, Some(&output));
        assert!(!rows.is_empty());
        assert!(rows[0].gross_income.is_none());
        assert!(rows[0].opening_balance.is_some());
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
        }];
        let csv = rows_to_csv(&rows);
        let data_line = csv.lines().nth(1).unwrap();
        let fields: Vec<&str> = data_line.split(',').collect();
        assert_eq!(fields[0], "1");
        assert_eq!(fields[1], "");
        assert_eq!(fields[7], "500000.00");
    }
}
