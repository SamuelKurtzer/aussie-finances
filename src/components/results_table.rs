use leptos::*;

use crate::domain::types::CalculatorOutput;

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
                        <td>"HELP/HECS Repayment"</td>
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
                        <td>"Effective Tax Rate"</td>
                        <td>{format!("{:.2}%", result.effective_tax_rate_percent)}</td>
                        <td>"-"</td>
                    </tr>
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
