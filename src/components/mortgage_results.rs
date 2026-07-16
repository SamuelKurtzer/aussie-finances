use leptos::*;

use crate::components::amortization_table::AmortizationTable;
use crate::components::line_chart::{ChartLine, MultiLineChart};
use crate::components::mortgage_summary::MortgageSummaryView;
use crate::domain::mortgages::{
    build_monthly_payment_series, MortgagePortfolioOutput, MortgageValidationError,
};
use crate::domain::types::DomainError;
use crate::formatting::chart_colors;

#[component]
pub fn MortgageResults(result: Result<MortgagePortfolioOutput, DomainError>) -> impl IntoView {
    match result {
        Ok(output) => {
            let warnings = output.warnings.clone();
            let balance = output.chart_series.total_balance.clone();
            let worst_balance = output.chart_series.worst_case_total_balance.clone();
            let total_repaid = output.chart_series.cumulative_repayment.clone();
            let worst_total_repaid = output.chart_series.worst_case_cumulative_repayment.clone();
            let interest = output.chart_series.cumulative_interest.clone();
            let worst_interest = output.chart_series.worst_case_cumulative_interest.clone();
            let offset = output.chart_series.offset_balance.clone();
            let period_months = output.chart_series.period_months.clone();
            let rows = output.amortization_rows.clone();
            let worst_rows = output.worst_case_amortization_rows.clone();
            let baseline_initial_offset = offset.first().copied().unwrap_or(0.0);
            let (monthly_months, monthly_principal, monthly_interest, monthly_offset) =
                build_monthly_payment_series(&rows, &period_months, baseline_initial_offset);
            let (_, worst_monthly_principal, worst_monthly_interest, worst_monthly_offset) =
                build_monthly_payment_series(&worst_rows, &period_months, 0.0);
            let monthly_total_repayment = monthly_principal
                .iter()
                .zip(monthly_interest.iter())
                .map(|(p, i)| p + i)
                .collect::<Vec<_>>();
            let worst_monthly_total_repayment = worst_monthly_principal
                .iter()
                .zip(worst_monthly_interest.iter())
                .map(|(p, i)| p + i)
                .collect::<Vec<_>>();

            view! {
                <section>
                    <MortgageSummaryView output=output />
                    {(!warnings.is_empty()).then(|| view! {
                        <section>
                            <h3>"Warnings"</h3>
                            <ul>
                                {warnings
                                    .iter()
                                    .map(|w| view! { <li class="muted">{w}</li> })
                                    .collect_view()}
                            </ul>
                        </section>
                    })}
                    <section class="chart-grid">
                        <MultiLineChart
                            title="Mortgage Trends Over Time".to_string()
                            period_months=period_months.clone()
                            lines=vec![
                                ChartLine {
                                    name: "Total Balance".to_string(),
                                    color: chart_colors::BALANCE,
                                    values: balance,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Total Balance (Worst Case)".to_string(),
                                    color: chart_colors::BALANCE,
                                    values: worst_balance,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                                ChartLine {
                                    name: "Cumulative Interest".to_string(),
                                    color: chart_colors::INTEREST,
                                    values: interest,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Cumulative Interest (Worst Case)".to_string(),
                                    color: chart_colors::INTEREST,
                                    values: worst_interest,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                                ChartLine {
                                    name: "Offset Balance".to_string(),
                                    color: chart_colors::OFFSET,
                                    values: offset,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Total Repaid".to_string(),
                                    color: chart_colors::REPAID,
                                    values: total_repaid,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Total Repaid (Worst Case)".to_string(),
                                    color: chart_colors::REPAID,
                                    values: worst_total_repaid,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                            ]
                        />
                        <MultiLineChart
                            title="Monthly Principal, Interest and Offset Top-Up".to_string()
                            period_months=monthly_months
                            lines=vec![
                                ChartLine {
                                    name: "Principal (Monthly)".to_string(),
                                    color: chart_colors::BALANCE,
                                    values: monthly_principal,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Principal (Worst Case)".to_string(),
                                    color: chart_colors::BALANCE,
                                    values: worst_monthly_principal,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                                ChartLine {
                                    name: "Interest (Monthly)".to_string(),
                                    color: chart_colors::INTEREST,
                                    values: monthly_interest,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Interest (Worst Case)".to_string(),
                                    color: chart_colors::INTEREST,
                                    values: worst_monthly_interest,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                                ChartLine {
                                    name: "Offset Top-Up (Monthly)".to_string(),
                                    color: chart_colors::OFFSET,
                                    values: monthly_offset,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Offset Top-Up (Worst Case)".to_string(),
                                    color: chart_colors::OFFSET,
                                    values: worst_monthly_offset,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                                ChartLine {
                                    name: "Total Repayment (Monthly)".to_string(),
                                    color: chart_colors::REPAID,
                                    values: monthly_total_repayment,
                                    opacity: 1.0,
                                    dashed: false,
                                },
                                ChartLine {
                                    name: "Total Repayment (Worst Case)".to_string(),
                                    color: chart_colors::REPAID,
                                    values: worst_monthly_total_repayment,
                                    opacity: 0.35,
                                    dashed: true,
                                },
                            ]
                        />
                    </section>
                    <AmortizationTable rows=rows period_months=period_months />
                </section>
            }
            .into_view()
        }
        Err(MortgageValidationError::Validation(issues)) => view! {
            <section>
                <h3>"Validation issues"</h3>
                <ul>
                    {issues
                        .iter()
                        .map(|i| {
                            view! { <li class="error">{format!("{}: {}", i.field, i.message)}</li> }
                        })
                        .collect_view()}
                </ul>
            </section>
        }
        .into_view(),
    }
}
