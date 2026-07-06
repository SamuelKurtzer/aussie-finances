use leptos::*;

use crate::components::line_chart::{ChartLine, MultiLineChart};
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, DebtRecycleInput, MortgagePortfolioInput, MortgageValidationError,
};
use crate::formatting::{chart_colors, fmt_int_commas, fmt_money};
use crate::storage::{
    load_from_storage, save_to_storage, DEBT_RECYCLE_STORAGE_KEY, MORTGAGE_STORAGE_KEY,
};

#[component]
pub fn DebtRecyclingPage() -> impl IntoView {
    let strategy = create_rw_signal(
        load_from_storage::<DebtRecycleInput>(DEBT_RECYCLE_STORAGE_KEY).unwrap_or_default(),
    );

    create_effect(move |_| {
        save_to_storage(DEBT_RECYCLE_STORAGE_KEY, &strategy.get());
    });

    let result = create_memo(move |_| {
        let mut portfolio =
            load_from_storage::<MortgagePortfolioInput>(MORTGAGE_STORAGE_KEY).unwrap_or_default();
        let mut recycle = strategy.get();
        recycle.normalize_mortgage_selection(&portfolio);
        portfolio.debt_recycle = Some(recycle);
        calculate_mortgage_portfolio(&portfolio, None)
    });

    view! {
        <section>
            <h2>"Debt Recycling"</h2>
            <p class="muted">
                "Each month, pay a fixed amount from your offset into the loan and redraw it to invest (non-split redraw). \
                The loan becomes mixed-purpose: interest is apportioned pro-rata between deductible and non-deductible \
                portions, and principal repayments erode the deductible share (contamination)."
            </p>

            <section class="field-group">
                <h3>"Strategy Inputs"</h3>
                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || strategy.get().enabled
                        on:change=move |ev| strategy.update(|s| s.enabled = event_target_checked(&ev))
                    />
                    <span>"Enable debt recycling strategy"</span>
                </label>

                <label>"Target mortgage"</label>
                <select
                    on:change=move |ev| {
                        let id = event_target_value(&ev).parse::<u32>().unwrap_or(0);
                        strategy.update(|s| s.mortgage_id = id);
                    }
                >
                    <For
                        each=move || {
                            load_from_storage::<MortgagePortfolioInput>(MORTGAGE_STORAGE_KEY).unwrap_or_default().mortgages
                        }
                        key=|m| m.id
                        children=move |m| {
                            let id = m.id;
                            view! {
                                <option value={id.to_string()} selected=move || strategy.get().mortgage_id == id>
                                    {format!("{} (#{})", m.name, id)}
                                </option>
                            }
                        }
                    />
                </select>

                <div class="three-col">
                    <div>
                        <label>"Monthly redraw amount (AUD)"</label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            prop:value=move || strategy.get().monthly_redraw_aud
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0);
                                strategy.update(|s| s.monthly_redraw_aud = value);
                            }
                        />
                    </div>
                    <div>
                        <label>"Emergency buffer (AUD)"</label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            prop:value=move || strategy.get().emergency_buffer_aud
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0);
                                strategy.update(|s| s.emergency_buffer_aud = value);
                            }
                        />
                    </div>
                    <div>
                        <label>"Starting investment value (AUD)"</label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            prop:value=move || strategy.get().starting_investment_aud
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0);
                                strategy.update(|s| s.starting_investment_aud = value);
                            }
                        />
                    </div>
                </div>

                <div class="four-col">
                    <div>
                        <label>"Growth rate (%)"</label>
                        <input
                            type="number"
                            min="0"
                            step="0.01"
                            prop:value=move || strategy.get().growth_rate_percent
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0);
                                strategy.update(|s| s.growth_rate_percent = value);
                            }
                        />
                    </div>
                    <div>
                        <label>"Dividend yield (%)"</label>
                        <input
                            type="number"
                            min="0"
                            step="0.01"
                            prop:value=move || strategy.get().dividend_yield_percent
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0);
                                strategy.update(|s| s.dividend_yield_percent = value);
                            }
                        />
                    </div>
                    <div>
                        <label>"Franking (%)"</label>
                        <input
                            type="number"
                            min="0"
                            max="100"
                            step="0.1"
                            prop:value=move || strategy.get().franking_percent
                            on:input=move |ev| {
                                let value = event_target_value(&ev)
                                    .parse::<f64>()
                                    .unwrap_or(0.0)
                                    .clamp(0.0, 100.0);
                                strategy.update(|s| s.franking_percent = value);
                            }
                        />
                    </div>
                    <div>
                        <label>"Company tax rate (%)"</label>
                        <input
                            type="number"
                            min="0.01"
                            max="99.99"
                            step="0.01"
                            prop:value=move || strategy.get().company_tax_rate_percent
                            on:input=move |ev| {
                                let value = event_target_value(&ev)
                                    .parse::<f64>()
                                    .unwrap_or(30.0)
                                    .clamp(0.01, 99.99);
                                strategy.update(|s| s.company_tax_rate_percent = value);
                            }
                        />
                    </div>
                </div>
            </section>

            {move || {
                match result.get() {
                    Ok(output) => {
                        let debt_recycle = output.debt_recycle.clone();
                        if let Some(dr) = debt_recycle {
                            let period_months = output.chart_series.period_months.clone();
                            let investment_values = dr.periods.iter().map(|p| p.investment_value).collect::<Vec<_>>();
                            let recycled_debt = dr
                                .periods
                                .iter()
                                .map(|p| p.recycled_debt_balance)
                                .collect::<Vec<_>>();
                            let deductible_interest = dr
                                .periods
                                .iter()
                                .map(|p| p.cumulative_deductible_interest)
                                .collect::<Vec<_>>();
                            let offset_series = output.chart_series.offset_balance.clone();

                            view! {
                                <section>
                                    <h3>"Debt Recycle Summary"</h3>
                                    <div class="summary-grid">
                                        <article class="mini-card">
                                            <span class="muted">"Total Redrawn"</span>
                                            <strong>{fmt_money(dr.summary.total_drawn)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Redraw Count"</span>
                                            <strong>{fmt_int_commas(dr.summary.draw_count as i64)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Ending Investment"</span>
                                            <strong>{fmt_money(dr.summary.ending_investment_value)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Ending Recycled Debt"</span>
                                            <strong>{fmt_money(dr.summary.ending_recycled_debt_balance)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Total Dividends"</span>
                                            <strong>{fmt_money(dr.summary.total_dividends)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Total Franking Credits"</span>
                                            <strong>{fmt_money(dr.summary.total_franking_credits)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Deductible Interest (Total)"</span>
                                            <strong>{fmt_money(dr.summary.total_deductible_interest)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Contamination (Recycled Principal Repaid)"</span>
                                            <strong>{fmt_money(dr.summary.recycled_principal_repaid)}</strong>
                                        </article>
                                    </div>

                                    <MultiLineChart
                                        title="Debt Recycling Projection".to_string()
                                        period_months=period_months
                                        lines=vec![
                                            ChartLine {
                                                name: "Offset Balance".to_string(),
                                                color: chart_colors::DR_OFFSET,
                                                values: offset_series,
                                                opacity: 1.0,
                                                dashed: false,
                                            },
                                            ChartLine {
                                                name: "Investment Value".to_string(),
                                                color: chart_colors::DR_INVESTMENT,
                                                values: investment_values,
                                                opacity: 1.0,
                                                dashed: false,
                                            },
                                            ChartLine {
                                                name: "Recycled (Deductible) Debt".to_string(),
                                                color: chart_colors::DR_RECYCLED_DEBT,
                                                values: recycled_debt,
                                                opacity: 1.0,
                                                dashed: false,
                                            },
                                            ChartLine {
                                                name: "Cumulative Deductible Interest".to_string(),
                                                color: chart_colors::DR_DEDUCTIBLE,
                                                values: deductible_interest,
                                                opacity: 0.9,
                                                dashed: true,
                                            },
                                        ]
                                    />

                                    {if output.warnings.is_empty() {
                                        view! { <p class="muted">"No projection warnings."</p> }.into_view()
                                    } else {
                                        view! {
                                            <ul class="warning-list">
                                                {output
                                                    .warnings
                                                    .iter()
                                                    .map(|w| view! { <li>{w.clone()}</li> })
                                                    .collect_view()}
                                            </ul>
                                        }.into_view()
                                    }}

                                    <h3>"Monthly Redraw Events"</h3>
                                    <div class="table-wrap">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th>"Period"</th>
                                                    <th>"Redraw"</th>
                                                    <th>"Offset Before"</th>
                                                    <th>"Offset After"</th>
                                                    <th>"Recycled Debt"</th>
                                                    <th>"Deductible Interest (Cum.)"</th>
                                                    <th>"Dividend"</th>
                                                    <th>"Franking"</th>
                                                    <th>"Investment"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {dr
                                                    .periods
                                                    .iter()
                                                    .filter(|p| p.redraw_amount > 0.0)
                                                    .map(|p| {
                                                        view! {
                                                            <tr>
                                                                <td>{format!("P{}", fmt_int_commas(p.period_index as i64))}</td>
                                                                <td>{fmt_money(p.redraw_amount)}</td>
                                                                <td>{fmt_money(p.offset_before)}</td>
                                                                <td>{fmt_money(p.offset_after)}</td>
                                                                <td>{fmt_money(p.recycled_debt_balance)}</td>
                                                                <td>{fmt_money(p.cumulative_deductible_interest)}</td>
                                                                <td>{fmt_money(p.dividend_cash)}</td>
                                                                <td>{fmt_money(p.franking_credit)}</td>
                                                                <td>{fmt_money(p.investment_value)}</td>
                                                            </tr>
                                                        }
                                                    })
                                                    .collect_view()}
                                            </tbody>
                                        </table>
                                    </div>
                                </section>
                            }.into_view()
                        } else {
                            view! { <p class="muted">"Enable debt recycling to run a projection."</p> }.into_view()
                        }
                    }
                    Err(MortgageValidationError::Validation(issues)) => {
                        view! {
                            <section class="panel">
                                <h3>"Validation Issues"</h3>
                                <ul class="warning-list">
                                    {issues
                                        .iter()
                                        .map(|issue| view! { <li>{format!("{}: {}", issue.field, issue.message)}</li> })
                                        .collect_view()}
                                </ul>
                            </section>
                        }.into_view()
                    }
                }
            }}
        </section>
    }
}
