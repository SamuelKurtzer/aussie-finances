use leptos::*;

use crate::domain::types::CalculatorOutput;
use crate::formatting::{fmt_currency, fmt_money};

#[component]
pub fn ResultsTable(result: CalculatorOutput) -> impl IntoView {
    view! {
        <section>
            <h2>"Estimated Results"</h2>
            <p class="muted">
                "Breakdown shown annually and per " {result.pay_frequency.to_string()} "."
            </p>
            <table>
                <thead>
                    <tr>
                        <th>"Metric"</th>
                        <th>"Annual"</th>
                        <th>{result.pay_frequency.to_string()}</th>
                    </tr>
                </thead>
                <tbody>
                    <tr>
                        <td>"Gross Income"</td>
                        <td>{fmt_money(result.gross_income_annual)}</td>
                        <td>{fmt_money(result.gross_income_period)}</td>
                    </tr>
                    <tr>
                        <td>"Taxable Income"</td>
                        <td>{fmt_money(result.taxable_income_annual)}</td>
                        <td>{fmt_money(result.taxable_income_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td>"Income Tax"</td>
                        <td>{fmt_money(result.income_tax_annual)}</td>
                        <td>{fmt_money(result.income_tax_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td>"LITO (offset)"</td>
                        <td>{format!("-{}", fmt_money(result.lito_annual))}</td>
                        <td>{format!("-{}", fmt_money(result.lito_annual / result.pay_frequency.periods_per_year()))}</td>
                    </tr>
                    <tr>
                        <td>"SAPTO (offset)"</td>
                        <td>{format!("-{}", fmt_money(result.sapto_annual))}</td>
                        <td>{format!("-{}", fmt_money(result.sapto_annual / result.pay_frequency.periods_per_year()))}</td>
                    </tr>
                    <tr>
                        <td>"Medicare Levy"</td>
                        <td>{fmt_money(result.medicare_levy_annual)}</td>
                        <td>{fmt_money(result.medicare_levy_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td>"Medicare Levy Surcharge"</td>
                        <td>{fmt_money(result.medicare_levy_surcharge_annual)}</td>
                        <td>{fmt_money(result.medicare_levy_surcharge_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td>"Study Loan Repayment"</td>
                        <td>{fmt_money(result.help_repayment_annual)}</td>
                        <td>{fmt_money(result.help_repayment_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td><strong>"Total Withheld"</strong></td>
                        <td><strong>{fmt_money(result.total_withheld_annual)}</strong></td>
                        <td><strong>{fmt_money(result.total_withheld_annual / result.pay_frequency.periods_per_year())}</strong></td>
                    </tr>
                    <tr>
                        <td><strong>"Net Income"</strong></td>
                        <td><strong>{fmt_money(result.net_income_annual)}</strong></td>
                        <td><strong>{fmt_money(result.net_income_period)}</strong></td>
                    </tr>
                    <tr>
                        <td>"Employer Super (SG)"</td>
                        <td>{fmt_money(result.super_guarantee_annual)}</td>
                        <td>{fmt_money(result.super_guarantee_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td>"Concessional Contributions"</td>
                        <td>{fmt_money(result.concessional_contributions_annual)}</td>
                        <td>{fmt_money(result.concessional_contributions_annual / result.pay_frequency.periods_per_year())}</td>
                    </tr>
                    <tr>
                        <td>"Division 293 (est., separate)"</td>
                        <td>{fmt_money(result.division_293_annual)}</td>
                        <td>"-"</td>
                    </tr>
                    <tr>
                        <td>"Effective Tax Rate"</td>
                        <td>{format!("{:.2}%", result.effective_tax_rate_percent)}</td>
                        <td>"-"</td>
                    </tr>
                    <tr>
                        <td>"Marginal Tax Rate"</td>
                        <td>{format!("{:.0}%", result.marginal_rate_percent)}</td>
                        <td>"-"</td>
                    </tr>
                </tbody>
            </table>
            {(!result.warnings.is_empty()).then(|| view! {
                <ul class="warning-list">
                    {result
                        .warnings
                        .iter()
                        .map(|w| view! { <li>{w.clone()}</li> })
                        .collect_view()}
                </ul>
            })}
            <h3>"Tax Bracket Breakdown"</h3>
            <table>
                <thead>
                    <tr>
                        <th>"Bracket"</th>
                        <th>"Rate"</th>
                        <th>"Tax"</th>
                    </tr>
                </thead>
                <tbody>
                    {result
                        .bracket_breakdown
                        .iter()
                        .map(|b| {
                            let range = match b.upper_bound {
                                Some(upper) => format!(
                                    "{} - {}",
                                    fmt_currency(b.lower_bound),
                                    fmt_currency(upper)
                                ),
                                None => format!("{}+", fmt_currency(b.lower_bound)),
                            };
                            view! {
                                <tr>
                                    <td>{range}</td>
                                    <td>{format!("{:.0}%", b.rate * 100.0)}</td>
                                    <td>{fmt_money(b.tax_amount)}</td>
                                </tr>
                            }
                        })
                        .collect_view()}
                </tbody>
            </table>
            <h3>"Assumptions"</h3>
            <ul>
                {result
                    .assumptions
                    .iter()
                    .map(|item| view! { <li>{item}</li> })
                    .collect_view()}
            </ul>
            <p class="muted">
                "Financial disclaimer: this is a general estimate only and does not account for your full personal tax circumstances."
            </p>
        </section>
    }
}
