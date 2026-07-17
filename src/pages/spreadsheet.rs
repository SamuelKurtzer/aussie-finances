use leptos::*;

use crate::backup::trigger_download;
use crate::components::line_chart::{ChartLine, MultiLineChart};
use crate::domain::spreadsheet::{build_spreadsheet, rows_to_csv, SpreadsheetRow};
use crate::formatting::{chart_colors, fmt_int_commas, fmt_money};
use crate::loaders::{load_income_input, load_mortgage_input, load_mortgage_output};

fn cell(val: Option<f64>) -> String {
    match val {
        Some(v) => fmt_money(v),
        None => "-".to_string(),
    }
}

#[component]
pub fn SpreadsheetPage() -> impl IntoView {
    let rows = create_memo(move |_| {
        let income_input = load_income_input();
        let portfolio_input = load_mortgage_input();
        let mortgage = load_mortgage_output();
        build_spreadsheet(
            income_input.as_ref(),
            portfolio_input.as_ref(),
            mortgage.as_ref(),
        )
    });

    let download_csv = move |_| {
        let data = rows.get();
        if data.is_empty() {
            return;
        }
        let csv = rows_to_csv(&data);
        trigger_download(&csv, "aus-fin-spreadsheet.csv", "text/csv");
    };

    view! {
        <section>
            <h2>"Spreadsheet"</h2>
            <p class="muted">
                "Unified monthly view of income, mortgage, and debt recycling data. "
                <button type="button" on:click=download_csv>"Download CSV"</button>
            </p>

            {move || {
                let data = rows.get();
                if data.is_empty() {
                    view! {
                        <p class="muted">
                            "No data to display. Configure your income, mortgages, or debt recycling on the other tabs first."
                        </p>
                    }.into_view()
                } else {
                    view! {
                        <NetWorthChart rows=data.clone() />
                        <SpreadsheetTable rows=data />
                    }.into_view()
                }
            }}
        </section>
    }
}

#[component]
fn NetWorthChart(rows: Vec<SpreadsheetRow>) -> impl IntoView {
    if rows.len() < 2 || rows.iter().all(|r| r.net_worth.is_none()) {
        return ().into_view();
    }

    let months: Vec<f64> = std::iter::once(0.0)
        .chain(rows.iter().map(|r| r.month as f64))
        .collect();
    let series = |f: fn(&SpreadsheetRow) -> Option<f64>| -> Option<Vec<f64>> {
        rows.iter().any(|r| f(r).is_some()).then(|| {
            // Repeat the first value at month 0 so lines span the full axis.
            std::iter::once(f(&rows[0]).unwrap_or(0.0))
                .chain(rows.iter().map(|r| f(r).unwrap_or(0.0)))
                .collect()
        })
    };

    let mut lines = vec![ChartLine {
        name: "Net Worth".to_string(),
        color: chart_colors::BALANCE,
        values: series(|r| r.net_worth).unwrap_or_default(),
        opacity: 1.0,
        dashed: false,
    }];
    let optional = [
        ("Property", chart_colors::PROPERTY, {
            series(|r| r.property_value)
        }),
        ("Investments", chart_colors::DR_INVESTMENT, {
            series(|r| r.dr_investment)
        }),
        ("Super", chart_colors::SUPER, series(|r| r.super_balance)),
        ("Total Debt", chart_colors::INTEREST, {
            series(|r| r.closing_balance)
        }),
    ];
    for (name, color, values) in optional {
        if let Some(values) = values {
            lines.push(ChartLine {
                name: name.to_string(),
                color,
                values,
                opacity: 0.85,
                dashed: false,
            });
        }
    }

    view! {
        <MultiLineChart
            title="Net Worth Projection".to_string()
            period_months=months
            lines=lines
        />
    }
    .into_view()
}

#[component]
fn SpreadsheetTable(rows: Vec<SpreadsheetRow>) -> impl IntoView {
    view! {
        <div class="spreadsheet-wrap">
            <table>
                <thead>
                    <tr>
                        <th>"Month"</th>
                        <th>"Gross Income"</th>
                        <th>"Net Income"</th>
                        <th>"Income Tax"</th>
                        <th>"Medicare"</th>
                        <th>"HELP"</th>
                        <th>"Total Withheld"</th>
                        <th>"Opening Bal"</th>
                        <th>"Repayment"</th>
                        <th>"Interest"</th>
                        <th>"Principal"</th>
                        <th>"Closing Bal"</th>
                        <th>"Offset"</th>
                        <th>"Cum. Interest"</th>
                        <th>"DR Redraw"</th>
                        <th>"DR Investment"</th>
                        <th>"DR Dividend"</th>
                        <th>"DR Franking"</th>
                        <th>"DR Recycled Debt"</th>
                        <th>"DR Cum. Deductible Interest"</th>
                        <th>"Property Value"</th>
                        <th>"Super Balance"</th>
                        <th>"Net Worth"</th>
                    </tr>
                </thead>
                <tbody>
                    {rows
                        .iter()
                        .map(|row| {
                            view! {
                                <tr>
                                    <td>{format!("M{}", fmt_int_commas(row.month as i64))}</td>
                                    <td>{cell(row.gross_income)}</td>
                                    <td>{cell(row.net_income)}</td>
                                    <td>{cell(row.income_tax)}</td>
                                    <td>{cell(row.medicare)}</td>
                                    <td>{cell(row.help)}</td>
                                    <td>{cell(row.total_withheld)}</td>
                                    <td>{cell(row.opening_balance)}</td>
                                    <td>{cell(row.repayment)}</td>
                                    <td>{cell(row.interest)}</td>
                                    <td>{cell(row.principal)}</td>
                                    <td>{cell(row.closing_balance)}</td>
                                    <td>{cell(row.offset)}</td>
                                    <td>{cell(row.cumulative_interest)}</td>
                                    <td>{cell(row.dr_draw)}</td>
                                    <td>{cell(row.dr_investment)}</td>
                                    <td>{cell(row.dr_dividend)}</td>
                                    <td>{cell(row.dr_franking)}</td>
                                    <td>{cell(row.dr_recycled_debt)}</td>
                                    <td>{cell(row.dr_deductible_interest)}</td>
                                    <td>{cell(row.property_value)}</td>
                                    <td>{cell(row.super_balance)}</td>
                                    <td>{cell(row.net_worth)}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
