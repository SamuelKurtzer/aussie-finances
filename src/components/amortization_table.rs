use leptos::*;

use crate::components::collapsible::Collapsible;
use crate::domain::mortgages::AmortizationRow;
use crate::domain::spreadsheet::map_periods_to_months;
use crate::formatting::{fmt_int_commas, fmt_money};

#[component]
pub fn AmortizationTable(rows: Vec<AmortizationRow>, period_months: Vec<f64>) -> impl IntoView {
    let monthly_rows: Vec<(usize, AmortizationRow)> = map_periods_to_months(&rows, &period_months)
        .into_iter()
        .enumerate()
        .map(|(i, row)| (i + 1, row))
        .collect();

    view! {
        <section>
            <Collapsible title="Amortization">
            <div class="table-wrap">
                <table>
                    <thead>
                        <tr>
                            <th>"Month"</th>
                            <th>"Open"</th>
                            <th>"Repayment"</th>
                            <th>"Interest"</th>
                            <th>"Principal"</th>
                            <th>"Close"</th>
                        </tr>
                    </thead>
                    <tbody>
                        {monthly_rows
                            .iter()
                            .map(|(month, row)| {
                                view! {
                                    <tr>
                                        <td>{format!("M{}", fmt_int_commas(*month as i64))}</td>
                                        <td>{fmt_money(row.opening_balance)}</td>
                                        <td>{fmt_money(row.repayment)}</td>
                                        <td>{fmt_money(row.interest)}</td>
                                        <td>{fmt_money(row.principal)}</td>
                                        <td>{fmt_money(row.closing_balance)}</td>
                                    </tr>
                                }
                            })
                            .collect_view()}
                    </tbody>
                </table>
            </div>
            </Collapsible>
        </section>
    }
}
