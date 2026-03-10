use leptos::*;

use crate::domain::mortgages::AmortizationRow;

fn fmt_money(value: f64) -> String {
    let sign = if value < 0.0 { "-" } else { "" };
    let abs = value.abs();
    let whole = abs.trunc() as i64;
    let cents = ((abs - whole as f64) * 100.0).round() as i64;
    format!("{sign}${}.{:02}", fmt_int_commas(whole), cents)
}

fn fmt_int_commas(n: i64) -> String {
    let sign = if n < 0 { "-" } else { "" };
    let s = n.abs().to_string();
    let mut out = String::new();
    for (i, ch) in s.chars().rev().enumerate() {
        if i > 0 && i % 3 == 0 {
            out.push(',');
        }
        out.push(ch);
    }
    let grouped: String = out.chars().rev().collect();
    format!("{sign}{grouped}")
}

#[component]
pub fn AmortizationTable(rows: Vec<AmortizationRow>, period_months: Vec<f64>) -> impl IntoView {
    // Show monthly rows (M1..Mterm) even when repayment cadence is weekly/fortnightly.
    let monthly_rows: Vec<(usize, AmortizationRow)> = if rows.is_empty() {
        Vec::new()
    } else {
        let month_series = if period_months.len() == rows.len() + 1 {
            period_months
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
                (month, rows[idx].clone())
            })
            .collect()
    };

    view! {
        <section>
            <h3>"Amortization"</h3>
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
        </section>
    }
}
