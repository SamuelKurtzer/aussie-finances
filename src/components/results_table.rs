use leptos::*;

use crate::components::collapsible::Collapsible;
use crate::components::info_tip::InfoTip;
use crate::domain::types::CalculatorOutput;
use crate::formatting::{fmt_currency, fmt_money};

#[component]
pub fn ResultsTable(result: CalculatorOutput) -> impl IntoView {
    let ppy = result.pay_frequency.periods_per_year();
    let show = |v: f64| v > 0.005;
    let extra_concessional =
        result.concessional_contributions_annual - result.super_guarantee_annual;
    let taxable = result.taxable_income_annual;

    view! {
        <section>
            <h2>"Estimated Results"</h2>
            <p class="muted">
                "Breakdown shown annually and per " {result.pay_frequency.to_string()} ". Rows that do not apply to you are hidden."
            </p>
            <div class="table-scroll">
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
                    {((result.taxable_income_annual - result.gross_income_annual).abs() > 0.005).then(|| view! {
                        <tr>
                            <td>
                                "Taxable Income"
                                <InfoTip text="Gross income minus deductions, salary sacrifice, and extra concessional super. Tax and levies are calculated on this amount." />
                            </td>
                            <td>{fmt_money(result.taxable_income_annual)}</td>
                            <td>{fmt_money(result.taxable_income_annual / ppy)}</td>
                        </tr>
                    })}
                    <tr>
                        <td>
                            "Income Tax"
                            <InfoTip text="Tax on your taxable income using the marginal bracket rates for the selected year and residency, before offsets." />
                        </td>
                        <td>{fmt_money(result.income_tax_annual)}</td>
                        <td>{fmt_money(result.income_tax_annual / ppy)}</td>
                    </tr>
                    {show(result.lito_annual).then(|| view! {
                        <tr>
                            <td>
                                "LITO (offset)"
                                <InfoTip text="Low Income Tax Offset: up to $700 off your income tax, tapering away between $37,500 and $66,667 of taxable income. Applied automatically for residents." />
                            </td>
                            <td>{format!("-{}", fmt_money(result.lito_annual))}</td>
                            <td>{format!("-{}", fmt_money(result.lito_annual / ppy))}</td>
                        </tr>
                    })}
                    {show(result.sapto_annual).then(|| view! {
                        <tr>
                            <td>
                                "SAPTO (offset)"
                                <InfoTip text="Seniors and Pensioners Tax Offset: up to $2,230 off income tax for eligible seniors, tapering at 12.5c per dollar of taxable income above $32,279." />
                            </td>
                            <td>{format!("-{}", fmt_money(result.sapto_annual))}</td>
                            <td>{format!("-{}", fmt_money(result.sapto_annual / ppy))}</td>
                        </tr>
                    })}
                    {show(result.medicare_levy_annual).then(|| view! {
                        <tr>
                            <td>
                                "Medicare Levy"
                                <InfoTip text="2% of taxable income for most residents, funding the public health system. Reduced or nil below the low-income thresholds." />
                            </td>
                            <td>{fmt_money(result.medicare_levy_annual)}</td>
                            <td>{fmt_money(result.medicare_levy_annual / ppy)}</td>
                        </tr>
                    })}
                    {show(result.medicare_levy_surcharge_annual).then(|| view! {
                        <tr>
                            <td>
                                "Medicare Levy Surcharge"
                                <InfoTip text="An extra 1% to 1.5% levy on higher incomes without an appropriate level of private hospital cover. Taking out cover removes it." />
                            </td>
                            <td>{fmt_money(result.medicare_levy_surcharge_annual)}</td>
                            <td>{fmt_money(result.medicare_levy_surcharge_annual / ppy)}</td>
                        </tr>
                    })}
                    {show(result.help_repayment_annual).then(|| view! {
                        <tr>
                            <td>
                                "Study Loan Repayment"
                                <InfoTip text="Compulsory repayment of HELP/HECS, VET, SSL, TSL, or SFSS debt, withheld once your repayment income passes the threshold for the year." />
                            </td>
                            <td>{fmt_money(result.help_repayment_annual)}</td>
                            <td>{fmt_money(result.help_repayment_annual / ppy)}</td>
                        </tr>
                    })}
                    <tr>
                        <td>
                            <strong>"Total Withheld"</strong>
                            <InfoTip text="Income tax after offsets, plus Medicare levy, surcharge, and study loan repayment. What your employer withholds across the year." />
                        </td>
                        <td><strong>{fmt_money(result.total_withheld_annual)}</strong></td>
                        <td><strong>{fmt_money(result.total_withheld_annual / ppy)}</strong></td>
                    </tr>
                    <tr>
                        <td>
                            <strong>"Net Income"</strong>
                            <InfoTip text="Take-home pay: gross income minus salary sacrifice, extra super, and everything withheld." />
                        </td>
                        <td><strong>{fmt_money(result.net_income_annual)}</strong></td>
                        <td><strong>{fmt_money(result.net_income_period)}</strong></td>
                    </tr>
                    <tr>
                        <td>
                            "Employer Super (SG)"
                            <InfoTip text="Superannuation guarantee your employer pays into your fund on top of salary and bonus (overtime is excluded)." />
                        </td>
                        <td>{fmt_money(result.super_guarantee_annual)}</td>
                        <td>{fmt_money(result.super_guarantee_annual / ppy)}</td>
                    </tr>
                    {show(extra_concessional).then(|| view! {
                        <tr>
                            <td>
                                "Concessional Contributions"
                                <InfoTip text="Employer super plus salary sacrifice and extra personal before-tax super, taxed at 15% inside the fund. The annual cap is $30,000." />
                            </td>
                            <td>{fmt_money(result.concessional_contributions_annual)}</td>
                            <td>{fmt_money(result.concessional_contributions_annual / ppy)}</td>
                        </tr>
                    })}
                    {show(result.division_293_annual).then(|| view! {
                        <tr>
                            <td>
                                "Division 293 (est., separate)"
                                <InfoTip text="An extra 15% tax on concessional super contributions when income plus contributions exceed $250,000. Assessed separately by the ATO and usually paid from super, so it is not in Total Withheld." />
                            </td>
                            <td>{fmt_money(result.division_293_annual)}</td>
                            <td>"-"</td>
                        </tr>
                    })}
                    <tr>
                        <td>
                            "Effective Tax Rate"
                            <InfoTip text="Total tax, levies, and loan repayments as a share of your gross income." />
                        </td>
                        <td>{format!("{:.2}%", result.effective_tax_rate_percent)}</td>
                        <td>"-"</td>
                    </tr>
                    <tr>
                        <td>
                            "Marginal Tax Rate"
                            <InfoTip text="The bracket rate on your next dollar of taxable income (excluding Medicare and study loan effects)." />
                        </td>
                        <td>{format!("{:.0}%", result.marginal_rate_percent)}</td>
                        <td>"-"</td>
                    </tr>
                </tbody>
            </table>
            </div>
            {(!result.warnings.is_empty()).then(|| view! {
                <ul class="warning-list">
                    {result
                        .warnings
                        .iter()
                        .map(|w| view! { <li>{w.clone()}</li> })
                        .collect_view()}
                </ul>
            })}
            <Collapsible title="Tax Bracket Breakdown">
            <p class="muted">
                "How your taxable income fills each bracket. Only the income inside a bracket is taxed at that bracket's rate."
            </p>
            <div class="table-scroll">
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
                        .filter(|b| b.lower_bound < taxable)
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
            </div>
            </Collapsible>
            <details class="fine-print">
                <summary>"Assumptions"</summary>
                <ul>
                    {result
                        .assumptions
                        .iter()
                        .map(|item| view! { <li>{item}</li> })
                        .collect_view()}
                </ul>
            </details>
            <p class="muted">
                "Financial disclaimer: this is a general estimate only and does not account for your full personal tax circumstances."
            </p>
        </section>
    }
}
