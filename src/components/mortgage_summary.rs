use leptos::*;

use crate::domain::mortgages::{MortgagePortfolioOutput, MortgageSummary};
use crate::formatting::fmt_money;

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
                <article class="mini-card">
                    <span class="muted">"Recycled Debt (Ending)"</span>
                    <strong>
                        {output
                            .debt_recycle
                            .as_ref()
                            .map(|d| fmt_money(d.summary.ending_recycled_debt_balance))
                            .unwrap_or_else(|| "$0.00".to_string())}
                    </strong>
                </article>
                <article class="mini-card">
                    <span class="muted">"Recycle Redraw Count"</span>
                    <strong>
                        {output
                            .debt_recycle
                            .as_ref()
                            .map(|d| d.summary.draw_count.to_string())
                            .unwrap_or_else(|| "0".to_string())}
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
