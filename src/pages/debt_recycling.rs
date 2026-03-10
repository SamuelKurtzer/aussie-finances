use leptos::*;

use crate::components::line_chart::{ChartLine, MultiLineChart};
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, DebtRecycleInput, MortgagePortfolioInput, MortgageValidationError,
};

#[cfg(target_arch = "wasm32")]
const MORTGAGE_STORAGE_KEY: &str = "aus_fin_mortgage_calculator_v1";
#[cfg(target_arch = "wasm32")]
const DEBT_RECYCLE_STORAGE_KEY: &str = "aus_fin_debt_recycle_v1";

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

#[cfg(target_arch = "wasm32")]
fn load_saved_portfolio() -> MortgagePortfolioInput {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(raw)) = storage.get_item(MORTGAGE_STORAGE_KEY) {
                if let Ok(parsed) = serde_json::from_str::<MortgagePortfolioInput>(&raw) {
                    return parsed;
                }
            }
        }
    }
    MortgagePortfolioInput::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_saved_portfolio() -> MortgagePortfolioInput {
    MortgagePortfolioInput::default()
}

#[cfg(target_arch = "wasm32")]
fn load_saved_strategy() -> DebtRecycleInput {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(Some(raw)) = storage.get_item(DEBT_RECYCLE_STORAGE_KEY) {
                if let Ok(parsed) = serde_json::from_str::<DebtRecycleInput>(&raw) {
                    return parsed;
                }
            }
        }
    }
    DebtRecycleInput::default()
}

#[cfg(not(target_arch = "wasm32"))]
fn load_saved_strategy() -> DebtRecycleInput {
    DebtRecycleInput::default()
}

#[cfg(target_arch = "wasm32")]
fn persist_strategy(input: &DebtRecycleInput) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(raw) = serde_json::to_string(input) {
                let _ = storage.set_item(DEBT_RECYCLE_STORAGE_KEY, &raw);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_strategy(_input: &DebtRecycleInput) {}

fn normalize_mortgage_selection(
    strategy: &mut DebtRecycleInput,
    portfolio: &MortgagePortfolioInput,
) {
    if portfolio
        .mortgages
        .iter()
        .any(|m| m.id == strategy.mortgage_id)
    {
        return;
    }
    strategy.mortgage_id = portfolio.mortgages.first().map(|m| m.id).unwrap_or(0);
}

#[component]
pub fn DebtRecyclingPage() -> impl IntoView {
    let strategy = create_rw_signal(load_saved_strategy());

    create_effect(move |_| {
        persist_strategy(&strategy.get());
    });

    let result = create_memo(move |_| {
        let mut portfolio = load_saved_portfolio();
        let mut recycle = strategy.get();
        normalize_mortgage_selection(&mut recycle, &portfolio);
        portfolio.debt_recycle = Some(recycle);
        calculate_mortgage_portfolio(&portfolio, None)
    });

    view! {
        <section>
            <h2>"Debt Recycling"</h2>
            <p class="muted">
                "Run triggered offset sweeps into investment debt splits and project debt, offset, and portfolio outcomes."
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
                        each=move || load_saved_portfolio().mortgages
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
                        <label>"Trigger offset target (AUD)"</label>
                        <input
                            type="number"
                            min="0"
                            step="1"
                            prop:value=move || strategy.get().trigger_target_aud
                            on:input=move |ev| {
                                let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0).max(0.0);
                                strategy.update(|s| s.trigger_target_aud = value);
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
                            let draw_amounts = dr.periods.iter().map(|p| p.draw_amount).collect::<Vec<_>>();
                            let offset_series = output.chart_series.offset_balance.clone();

                            view! {
                                <section>
                                    <h3>"Debt Recycle Summary"</h3>
                                    <div class="summary-grid">
                                        <article class="mini-card">
                                            <span class="muted">"Total Drawn"</span>
                                            <strong>{fmt_money(dr.summary.total_drawn)}</strong>
                                        </article>
                                        <article class="mini-card">
                                            <span class="muted">"Draw Count"</span>
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
                                    </div>

                                    <MultiLineChart
                                        title="Debt Recycling Projection".to_string()
                                        period_months=period_months
                                        lines=vec![
                                            ChartLine {
                                                name: "Offset Balance".to_string(),
                                                color: "#4ade80",
                                                values: offset_series,
                                                opacity: 1.0,
                                                dashed: false,
                                            },
                                            ChartLine {
                                                name: "Investment Value".to_string(),
                                                color: "#38bdf8",
                                                values: investment_values,
                                                opacity: 1.0,
                                                dashed: false,
                                            },
                                            ChartLine {
                                                name: "Recycled Debt Balance".to_string(),
                                                color: "#fb7185",
                                                values: recycled_debt,
                                                opacity: 1.0,
                                                dashed: false,
                                            },
                                            ChartLine {
                                                name: "Period Draw Amount".to_string(),
                                                color: "#f59e0b",
                                                values: draw_amounts,
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

                                    <h3>"Draw Events"</h3>
                                    <div class="table-wrap">
                                        <table>
                                            <thead>
                                                <tr>
                                                    <th>"Period"</th>
                                                    <th>"Draw"</th>
                                                    <th>"New Split"</th>
                                                    <th>"Offset Before"</th>
                                                    <th>"Offset After"</th>
                                                    <th>"Dividend"</th>
                                                    <th>"Franking"</th>
                                                    <th>"Investment"</th>
                                                </tr>
                                            </thead>
                                            <tbody>
                                                {dr
                                                    .periods
                                                    .iter()
                                                    .filter(|p| p.draw_amount > 0.0)
                                                    .map(|p| {
                                                        view! {
                                                            <tr>
                                                                <td>{format!("P{}", fmt_int_commas(p.period_index as i64))}</td>
                                                                <td>{fmt_money(p.draw_amount)}</td>
                                                                <td>{p.new_split_id.map(|id| format!("#{id}")).unwrap_or_else(|| "-".to_string())}</td>
                                                                <td>{fmt_money(p.offset_before)}</td>
                                                                <td>{fmt_money(p.offset_after)}</td>
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
