use leptos::*;

use crate::domain::mortgages::{MortgagePortfolioOutput, MortgageSummary};

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
pub fn MortgageSummaryView(output: MortgagePortfolioOutput) -> impl IntoView {
    view! {
        <section>
            <h2>"Mortgage Summary"</h2>
            <div class="summary-grid">
                <article class="mini-card">
                    <span class="muted">"Total Debt"</span>
                    <strong>{fmt_money(output.portfolio_totals.total_debt)}</strong>
                </article>
                <article class="mini-card">
                    <span class="muted">"Total Equity"</span>
                    <strong>{fmt_money(output.portfolio_totals.total_equity)}</strong>
                </article>
                <article class="mini-card">
                    <span class="muted">"Portfolio LVR"</span>
                    <strong>{format!("{:.2}%", output.portfolio_totals.portfolio_lvr_percent)}</strong>
                </article>
                <article class="mini-card">
                    <span class="muted">"Repayment / Period"</span>
                    <strong>{fmt_money(output.portfolio_totals.periodic_repayment_total)}</strong>
                </article>
                <article class="mini-card">
                    <span class="muted">"Projected Interest"</span>
                    <strong>{fmt_money(output.portfolio_totals.projected_total_interest)}</strong>
                </article>
                <article class="mini-card">
                    <span class="muted">"Repayment % Net Income"</span>
                    <strong>
                        {output
                            .repayment_to_income_percent
                            .map(|v| format!("{v:.2}%"))
                            .unwrap_or_else(|| "N/A".to_string())}
                    </strong>
                </article>
            </div>

            <h3>"By Mortgage"</h3>
            <table>
                <thead>
                    <tr>
                        <th>"Name"</th>
                        <th>"Debt"</th>
                        <th>"Home Value"</th>
                        <th>"Equity"</th>
                        <th>"LVR"</th>
                    </tr>
                </thead>
                <tbody>
                    {output
                        .mortgage_summaries
                        .iter()
                        .map(|m: &MortgageSummary| {
                            view! {
                                <tr>
                                    <td>{m.mortgage_name.clone()}</td>
                                    <td>{fmt_money(m.debt)}</td>
                                    <td>{fmt_money(m.property_value)}</td>
                                    <td>{fmt_money(m.equity)}</td>
                                    <td>{format!("{:.2}%", m.lvr_percent)}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
        </section>
    }
}
