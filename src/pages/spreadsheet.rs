use leptos::*;

use crate::backup::trigger_download;
use crate::domain::spreadsheet::{build_spreadsheet, rows_to_csv, SpreadsheetRow};
use crate::formatting::{fmt_int_commas, fmt_money};
use crate::storage::{
    load_from_storage, load_raw_from_storage, DEBT_RECYCLE_STORAGE_KEY, INCOME_STORAGE_KEY,
    MORTGAGE_STORAGE_KEY,
};

fn cell(val: Option<f64>) -> String {
    match val {
        Some(v) => fmt_money(v),
        None => "-".to_string(),
    }
}

fn load_income_output() -> Option<crate::domain::types::CalculatorOutput> {
    let raw = load_raw_from_storage(INCOME_STORAGE_KEY)?;
    let input = serde_json::from_str::<crate::domain::types::CalculatorInput>(&raw).ok()?;
    let rules = crate::domain::tax_rules::TaxRules::for_year(input.financial_year);
    crate::domain::calculator::calculate_income(&input, &rules).ok()
}

fn load_mortgage_output() -> Option<crate::domain::mortgages::MortgagePortfolioOutput> {
    let raw = load_raw_from_storage(MORTGAGE_STORAGE_KEY)?;
    let mut portfolio =
        serde_json::from_str::<crate::domain::mortgages::MortgagePortfolioInput>(&raw).ok()?;

    portfolio.debt_recycle =
        load_from_storage::<crate::domain::mortgages::DebtRecycleInput>(DEBT_RECYCLE_STORAGE_KEY);

    let income_raw = load_raw_from_storage(INCOME_STORAGE_KEY);
    let income_ctx = income_raw
        .as_deref()
        .and_then(crate::domain::mortgages::load_income_context_from_saved_input);

    crate::domain::mortgages::calculate_mortgage_portfolio(&portfolio, income_ctx.as_ref()).ok()
}

#[component]
pub fn SpreadsheetPage() -> impl IntoView {
    let rows = create_memo(move |_| {
        let income = load_income_output();
        let mortgage = load_mortgage_output();
        build_spreadsheet(income.as_ref(), mortgage.as_ref())
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
                    view! { <SpreadsheetTable rows=data /> }.into_view()
                }
            }}
        </section>
    }
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
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </div>
    }
}
