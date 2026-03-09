use leptos::*;

use crate::components::amortization_table::AmortizationTable;
use crate::components::line_chart::{ChartLine, MultiLineChart};
use crate::components::mortgage_summary::MortgageSummaryView;
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, AmortizationRow, LoanPurpose, LoanRepaymentType, MortgageInput,
    MortgagePortfolioInput, MortgageValidationError, RateType, RepaymentCadence, SplitInput,
};

#[cfg(target_arch = "wasm32")]
const MORTGAGE_STORAGE_KEY: &str = "aus_fin_mortgage_calculator_v1";
#[cfg(target_arch = "wasm32")]
const INCOME_STORAGE_KEY: &str = "aus_fin_income_calculator_v1";

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
fn persist_portfolio(input: &MortgagePortfolioInput) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(serialized) = serde_json::to_string(input) {
                let _ = storage.set_item(MORTGAGE_STORAGE_KEY, &serialized);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn persist_portfolio(_input: &MortgagePortfolioInput) {}

#[cfg(target_arch = "wasm32")]
fn clear_saved_portfolio() {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(MORTGAGE_STORAGE_KEY);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
fn clear_saved_portfolio() {}

#[cfg(target_arch = "wasm32")]
fn load_income_context() -> Option<crate::domain::mortgages::IncomeContext> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let raw = storage.get_item(INCOME_STORAGE_KEY).ok()??;
    crate::domain::mortgages::load_income_context_from_saved_input(&raw)
}

#[cfg(not(target_arch = "wasm32"))]
fn load_income_context() -> Option<crate::domain::mortgages::IncomeContext> {
    None
}

fn next_mortgage_id(input: &MortgagePortfolioInput) -> u32 {
    input.mortgages.iter().map(|m| m.id).max().unwrap_or(0) + 1
}

fn next_split_id(mortgage: &MortgageInput) -> u32 {
    mortgage.splits.iter().map(|s| s.id).max().unwrap_or(0) + 1
}

fn build_monthly_payment_series(
    rows: &[AmortizationRow],
    period_months: &[f64],
    initial_offset_balance: f64,
) -> (Vec<f64>, Vec<f64>, Vec<f64>, Vec<f64>) {
    if rows.is_empty() {
        return (vec![0.0], vec![0.0], vec![0.0], vec![0.0]);
    }

    let month_series: Vec<f64> = if period_months.len() == rows.len() + 1 {
        period_months.to_vec()
    } else {
        (0..=rows.len()).map(|i| i as f64).collect()
    };

    let max_month = month_series.last().copied().unwrap_or(rows.len() as f64).ceil() as usize;
    let mut principal = vec![0.0_f64; max_month + 1];
    let mut interest = vec![0.0_f64; max_month + 1];
    let mut offset_top_up = vec![0.0_f64; max_month + 1];

    let mut previous_offset = initial_offset_balance.max(0.0);
    for (i, row) in rows.iter().enumerate() {
        let delta_offset = (row.offset_balance - previous_offset).max(0.0);
        previous_offset = row.offset_balance;

        let start = month_series.get(i).copied().unwrap_or(i as f64);
        let end = month_series.get(i + 1).copied().unwrap_or((i + 1) as f64);
        let duration = (end - start).max(1e-9);
        let principal_rate = row.principal / duration;
        let interest_rate = row.interest / duration;
        let offset_rate = delta_offset / duration;

        let first_month = start.floor().max(0.0) as usize + 1;
        let last_month = end.ceil().max(0.0) as usize;

        for month in first_month..=last_month.min(max_month) {
            let left = (month - 1) as f64;
            let right = month as f64;
            let overlap = (end.min(right) - start.max(left)).max(0.0);
            if overlap <= 0.0 {
                continue;
            }
            principal[month] += principal_rate * overlap;
            interest[month] += interest_rate * overlap;
            offset_top_up[month] += offset_rate * overlap;
        }
    }

    let months = (0..=max_month).map(|m| m as f64).collect::<Vec<_>>();
    (months, principal, interest, offset_top_up)
}

#[component]
pub fn MortgagesPage() -> impl IntoView {
    let portfolio = create_rw_signal(load_saved_portfolio());

    create_effect(move |_| {
        persist_portfolio(&portfolio.get());
    });

    let income_context = create_memo(move |_| load_income_context());
    let result = create_memo(move |_| {
        let income = income_context.get();
        calculate_mortgage_portfolio(&portfolio.get(), income.as_ref())
    });

    let add_mortgage = {
        let portfolio = portfolio;
        move |_| {
            portfolio.update(|p| {
                if p.mortgages.len() >= 10 {
                    return;
                }
                let id = next_mortgage_id(p);
                let mut mortgage = MortgageInput::default();
                mortgage.id = id;
                mortgage.name = format!("Mortgage {}", id);
                for (idx, split) in mortgage.splits.iter_mut().enumerate() {
                    split.id = (idx + 1) as u32;
                }
                p.mortgages.push(mortgage);
            });
        }
    };

    let reset = {
        let portfolio = portfolio;
        move |_| {
            clear_saved_portfolio();
            portfolio.set(MortgagePortfolioInput::default());
        }
    };

    view! {
        <section class="mortgage-layout">
            <div>
                <h2>"Mortgage Planner"</h2>
                <p class="muted">
                    "Model multiple mortgages and splits, project balances over time, and track repayment pressure against net income."
                </p>

                <section class="field-group">
                    <h3>"Portfolio Settings"</h3>
                    <label for="cadence">"Repayment cadence"</label>
                    <select
                        id="cadence"
                        on:change=move |ev| {
                            let value = event_target_value(&ev);
                            portfolio.update(|p| {
                                p.repayment_cadence = match value.as_str() {
                                    "weekly" => RepaymentCadence::Weekly,
                                    "fortnightly" => RepaymentCadence::Fortnightly,
                                    _ => RepaymentCadence::Monthly,
                                }
                            })
                        }
                    >
                        <option value="weekly" selected=move || portfolio.get().repayment_cadence == RepaymentCadence::Weekly>"Weekly"</option>
                        <option value="fortnightly" selected=move || portfolio.get().repayment_cadence == RepaymentCadence::Fortnightly>"Fortnightly"</option>
                        <option value="monthly" selected=move || portfolio.get().repayment_cadence == RepaymentCadence::Monthly>"Monthly"</option>
                    </select>

                    <label class="check-row">
                        <input
                            type="checkbox"
                            prop:checked=move || portfolio.get().match_income_cadence
                            on:change=move |ev| {
                                portfolio.update(|p| p.match_income_cadence = event_target_checked(&ev))
                            }
                        />
                        <span>"Use income cadence context for affordability metric"</span>
                    </label>

                    <label for="offset-top-up">"Offset top-up per period (AUD)"</label>
                    <input
                        id="offset-top-up"
                        type="number"
                        min="0"
                        step="1"
                        prop:value=move || portfolio.get().offset_top_up_per_period
                        on:input=move |ev| {
                            let value = event_target_value(&ev).parse::<f64>().unwrap_or(0.0);
                            portfolio.update(|p| p.offset_top_up_per_period = value.max(0.0));
                        }
                    />

                    <div class="button-row">
                        <button type="button" on:click=add_mortgage>"Add Mortgage"</button>
                        <button type="button" class="secondary" on:click=reset>"Reset"</button>
                    </div>
                </section>

                <For
                    each=move || portfolio.get().mortgages
                    key=|m| m.id
                    children=move |mortgage| {
                        let mortgage_id = mortgage.id;
                        view! {
                            <section class="field-group">
                                <div class="row-head">
                                    <h3>{move || format!("{}", mortgage.name.clone())}</h3>
                                    <button
                                        type="button"
                                        class="secondary"
                                        on:click=move |_| {
                                            portfolio.update(|p| {
                                                if p.mortgages.len() <= 1 {
                                                    return;
                                                }
                                                p.mortgages.retain(|m| m.id != mortgage_id);
                                            })
                                        }
                                    >
                                        "Remove"
                                    </button>
                                </div>

                                <label>"Mortgage name"</label>
                                <input
                                    type="text"
                                    prop:value=move || {
                                        portfolio
                                            .get()
                                            .mortgages
                                            .iter()
                                            .find(|m| m.id == mortgage_id)
                                            .map(|m| m.name.clone())
                                            .unwrap_or_default()
                                    }
                                    on:input=move |ev| {
                                        let value = event_target_value(&ev);
                                        portfolio.update(|p| {
                                            if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id)
                                            {
                                                m.name = value.clone();
                                            }
                                        });
                                    }
                                />

                                <div class="three-col">
                                    <div>
                                        <label>"Home value (AUD)"</label>
                                        <input
                                            type="number"
                                            min="0"
                                            step="1"
                                            prop:value=move || {
                                                portfolio
                                                    .get()
                                                    .mortgages
                                                    .iter()
                                                    .find(|m| m.id == mortgage_id)
                                                    .map(|m| m.home_value)
                                                    .unwrap_or(0.0)
                                            }
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev)
                                                    .parse::<f64>()
                                                    .unwrap_or(0.0)
                                                    .max(0.0);
                                                portfolio.update(|p| {
                                                    if let Some(m) = p
                                                        .mortgages
                                                        .iter_mut()
                                                        .find(|m| m.id == mortgage_id)
                                                    {
                                                        m.home_value = value;
                                                    }
                                                });
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label>"Offset balance (AUD)"</label>
                                        <input
                                            type="number"
                                            min="0"
                                            prop:value=move || {
                                                portfolio
                                                    .get()
                                                    .mortgages
                                                    .iter()
                                                    .find(|m| m.id == mortgage_id)
                                                    .map(|m| m.offset_balance)
                                                    .unwrap_or(0.0)
                                            }
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev)
                                                    .parse::<f64>()
                                                    .unwrap_or(0.0)
                                                    .max(0.0);
                                                portfolio.update(|p| {
                                                    if let Some(m) = p
                                                        .mortgages
                                                        .iter_mut()
                                                        .find(|m| m.id == mortgage_id)
                                                    {
                                                        m.offset_balance = value;
                                                    }
                                                });
                                            }
                                        />
                                    </div>
                                    <div>
                                        <label>"Loan length (months)"</label>
                                        <input
                                            type="number"
                                            min="1"
                                            step="1"
                                            prop:value=move || {
                                                portfolio
                                                    .get()
                                                    .mortgages
                                                    .iter()
                                                    .find(|m| m.id == mortgage_id)
                                                    .map(|m| m.term_months)
                                                    .unwrap_or(360)
                                            }
                                            on:input=move |ev| {
                                                let value = event_target_value(&ev)
                                                    .parse::<u32>()
                                                    .unwrap_or(360)
                                                    .max(1);
                                                portfolio.update(|p| {
                                                    if let Some(m) = p
                                                        .mortgages
                                                        .iter_mut()
                                                        .find(|m| m.id == mortgage_id)
                                                    {
                                                        m.term_months = value;
                                                    }
                                                });
                                            }
                                        />
                                    </div>
                                </div>

                                <div class="row-head">
                                    <h4>"Splits"</h4>
                                    <button
                                        type="button"
                                        on:click=move |_| {
                                            portfolio.update(|p| {
                                                if let Some(m) = p
                                                    .mortgages
                                                    .iter_mut()
                                                    .find(|m| m.id == mortgage_id)
                                                {
                                                    if m.splits.len() >= 10 {
                                                        return;
                                                    }
                                                    let id = next_split_id(m);
                                                    let mut split = SplitInput::default();
                                                    split.id = id;
                                                    split.name = format!("Split {}", id);
                                                    m.splits.push(split);
                                                }
                                            })
                                        }
                                    >
                                        "Add Split"
                                    </button>
                                </div>

                                <For
                                    each=move || {
                                        portfolio
                                            .get()
                                            .mortgages
                                            .iter()
                                            .find(|m| m.id == mortgage_id)
                                            .map(|m| m.splits.clone())
                                            .unwrap_or_default()
                                    }
                                    key=|s| s.id
                                    children=move |split| {
                                        let split_id = split.id;
                                        view! {
                                            <div class="split-block">
                                                <div class="row-head">
                                                    <h4>{move || split.name.clone()}</h4>
                                                    <button
                                                        type="button"
                                                        class="secondary"
                                                        on:click=move |_| {
                                                            portfolio.update(|p| {
                                                                if let Some(m) = p
                                                                    .mortgages
                                                                    .iter_mut()
                                                                    .find(|m| m.id == mortgage_id)
                                                                {
                                                                    if m.splits.len() <= 1 {
                                                                        return;
                                                                    }
                                                                    m.splits.retain(|s| s.id != split_id);
                                                                }
                                                            })
                                                        }
                                                    >
                                                        "Remove"
                                                    </button>
                                                </div>

                                                <div class="four-col">
                                                    <div>
                                                        <label>"Name"</label>
                                                        <input
                                                            type="text"
                                                            prop:value=move || {
                                                                portfolio
                                                                    .get()
                                                                    .mortgages
                                                                    .iter()
                                                                    .find(|m| m.id == mortgage_id)
                                                                    .and_then(|m| {
                                                                        m.splits
                                                                            .iter()
                                                                            .find(|s| s.id == split_id)
                                                                    })
                                                                    .map(|s| s.name.clone())
                                                                    .unwrap_or_default()
                                                            }
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.name = value.clone();
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </div>
                                                    <div>
                                                        <label>"Loan amount"</label>
                                                        <input
                                                            type="number"
                                                            min="0"
                                                            prop:value=move || {
                                                                portfolio
                                                                    .get()
                                                                    .mortgages
                                                                    .iter()
                                                                    .find(|m| m.id == mortgage_id)
                                                                    .and_then(|m| {
                                                                        m.splits
                                                                            .iter()
                                                                            .find(|s| s.id == split_id)
                                                                    })
                                                                    .map(|s| s.loan_amount)
                                                                    .unwrap_or(0.0)
                                                            }
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev)
                                                                    .parse::<f64>()
                                                                    .unwrap_or(0.0)
                                                                    .max(0.0);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.loan_amount = value;
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </div>
                                                    <div>
                                                        <label>"Rate (%)"</label>
                                                        <input
                                                            type="number"
                                                            min="0"
                                                            step="0.01"
                                                            prop:value=move || {
                                                                portfolio
                                                                    .get()
                                                                    .mortgages
                                                                    .iter()
                                                                    .find(|m| m.id == mortgage_id)
                                                                    .and_then(|m| {
                                                                        m.splits
                                                                            .iter()
                                                                            .find(|s| s.id == split_id)
                                                                    })
                                                                    .map(|s| s.annual_rate_percent)
                                                                    .unwrap_or(0.0)
                                                            }
                                                            on:change=move |ev| {
                                                                let value = event_target_value(&ev)
                                                                    .parse::<f64>()
                                                                    .unwrap_or(0.0)
                                                                    .max(0.0);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.annual_rate_percent = value;
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </div>
                                                    <div>
                                                        <label>"IO years"</label>
                                                        <input
                                                            type="number"
                                                            min="0"
                                                            step="0.5"
                                                            prop:value=move || {
                                                                portfolio
                                                                    .get()
                                                                    .mortgages
                                                                    .iter()
                                                                    .find(|m| m.id == mortgage_id)
                                                                    .and_then(|m| {
                                                                        m.splits
                                                                            .iter()
                                                                            .find(|s| s.id == split_id)
                                                                    })
                                                                    .map(|s| s.interest_only_years)
                                                                    .unwrap_or(0.0)
                                                            }
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev)
                                                                    .parse::<f64>()
                                                                    .unwrap_or(0.0)
                                                                    .max(0.0);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.interest_only_years = value;
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </div>
                                                </div>

                                                <div class="three-col">
                                                    <div>
                                                        <label>"Rate type"</label>
                                                        <select
                                                            on:change=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.rate_type = if value == "fixed" {
                                                                            RateType::Fixed
                                                                        } else {
                                                                            RateType::Variable
                                                                        };
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <option
                                                                value="variable"
                                                                selected=move || {
                                                                    portfolio
                                                                        .get()
                                                                        .mortgages
                                                                        .iter()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                        .map(|s| s.rate_type == RateType::Variable)
                                                                        .unwrap_or(true)
                                                                }
                                                            >
                                                                "Variable"
                                                            </option>
                                                            <option
                                                                value="fixed"
                                                                selected=move || {
                                                                    portfolio
                                                                        .get()
                                                                        .mortgages
                                                                        .iter()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                        .map(|s| s.rate_type == RateType::Fixed)
                                                                        .unwrap_or(false)
                                                                }
                                                            >
                                                                "Fixed"
                                                            </option>
                                                        </select>
                                                    </div>
                                                    <div>
                                                        <label>"Purpose"</label>
                                                        <select
                                                            on:change=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.loan_purpose = if value == "investment" {
                                                                            LoanPurpose::Investment
                                                                        } else {
                                                                            LoanPurpose::OwnerOccupied
                                                                        };
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <option
                                                                value="owner"
                                                                selected=move || {
                                                                    portfolio
                                                                        .get()
                                                                        .mortgages
                                                                        .iter()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                        .map(|s| s.loan_purpose == LoanPurpose::OwnerOccupied)
                                                                        .unwrap_or(true)
                                                                }
                                                            >
                                                                "Owner Occupied"
                                                            </option>
                                                            <option
                                                                value="investment"
                                                                selected=move || {
                                                                    portfolio
                                                                        .get()
                                                                        .mortgages
                                                                        .iter()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                        .map(|s| s.loan_purpose == LoanPurpose::Investment)
                                                                        .unwrap_or(false)
                                                                }
                                                            >
                                                                "Investment"
                                                            </option>
                                                        </select>
                                                    </div>
                                                    <div>
                                                        <label>"Repayment type"</label>
                                                        <select
                                                            on:change=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                portfolio.update(|p| {
                                                                    if let Some(s) = p
                                                                        .mortgages
                                                                        .iter_mut()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter_mut()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                    {
                                                                        s.repayment_type = if value == "io" {
                                                                            LoanRepaymentType::InterestOnlyThenPrincipalAndInterest
                                                                        } else {
                                                                            LoanRepaymentType::PrincipalAndInterest
                                                                        };
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            <option
                                                                value="pi"
                                                                selected=move || {
                                                                    portfolio
                                                                        .get()
                                                                        .mortgages
                                                                        .iter()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                        .map(|s| {
                                                                            s.repayment_type
                                                                                == LoanRepaymentType::PrincipalAndInterest
                                                                        })
                                                                        .unwrap_or(true)
                                                                }
                                                            >
                                                                "Principal & Interest"
                                                            </option>
                                                            <option
                                                                value="io"
                                                                selected=move || {
                                                                    portfolio
                                                                        .get()
                                                                        .mortgages
                                                                        .iter()
                                                                        .find(|m| m.id == mortgage_id)
                                                                        .and_then(|m| {
                                                                            m.splits
                                                                                .iter()
                                                                                .find(|s| s.id == split_id)
                                                                        })
                                                                        .map(|s| {
                                                                            s.repayment_type
                                                                                == LoanRepaymentType::InterestOnlyThenPrincipalAndInterest
                                                                        })
                                                                        .unwrap_or(false)
                                                                }
                                                            >
                                                                "Interest Only -> P&I"
                                                            </option>
                                                        </select>
                                                    </div>
                                                </div>
                                            </div>
                                        }
                                    }
                                />
                            </section>
                        }
                    }
                />
            </div>

            <div>
                {move || {
                    match result.get() {
                        Ok(output) => {
                            let warnings = output.warnings.clone();
                            let balance = output.chart_series.total_balance.clone();
                            let worst_balance = output.chart_series.worst_case_total_balance.clone();
                            let total_repaid = output.chart_series.cumulative_repayment.clone();
                            let worst_total_repaid =
                                output.chart_series.worst_case_cumulative_repayment.clone();
                            let interest = output.chart_series.cumulative_interest.clone();
                            let worst_interest =
                                output.chart_series.worst_case_cumulative_interest.clone();
                            let offset = output.chart_series.offset_balance.clone();
                            let period_months = output.chart_series.period_months.clone();
                            let rows = output.amortization_rows.clone();
                            let worst_rows = output.worst_case_amortization_rows.clone();
                            let baseline_initial_offset = offset.first().copied().unwrap_or(0.0);
                            let (monthly_months, monthly_principal, monthly_interest, monthly_offset) =
                                build_monthly_payment_series(&rows, &period_months, baseline_initial_offset);
                            let (_, worst_monthly_principal, worst_monthly_interest, worst_monthly_offset) =
                                build_monthly_payment_series(&worst_rows, &period_months, 0.0);
                            let monthly_total_repayment = monthly_principal
                                .iter()
                                .zip(monthly_interest.iter())
                                .map(|(p, i)| p + i)
                                .collect::<Vec<_>>();
                            let worst_monthly_total_repayment = worst_monthly_principal
                                .iter()
                                .zip(worst_monthly_interest.iter())
                                .map(|(p, i)| p + i)
                                .collect::<Vec<_>>();

                            view! {
                                <section>
                                    <MortgageSummaryView output=output />
                                    {if warnings.is_empty() {
                                        view! { <></> }.into_view()
                                    } else {
                                        view! {
                                            <section>
                                                <h3>"Warnings"</h3>
                                                <ul>
                                                    {warnings
                                                        .iter()
                                                        .map(|w| view! { <li class="muted">{w}</li> })
                                                        .collect_view()}
                                                </ul>
                                            </section>
                                        }
                                            .into_view()
                                    }}
                                    <section class="chart-grid">
                                        <MultiLineChart
                                            title="Mortgage Trends Over Time".to_string()
                                            period_months=period_months.clone()
                                            lines=vec![
                                                ChartLine {
                                                    name: "Total Balance".to_string(),
                                                    color: "#25c59a",
                                                    values: balance,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Total Balance (Worst Case)".to_string(),
                                                    color: "#25c59a",
                                                    values: worst_balance,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                                ChartLine {
                                                    name: "Cumulative Interest".to_string(),
                                                    color: "#5aa9ff",
                                                    values: interest,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Cumulative Interest (Worst Case)".to_string(),
                                                    color: "#5aa9ff",
                                                    values: worst_interest,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                                ChartLine {
                                                    name: "Offset Balance".to_string(),
                                                    color: "#f59f42",
                                                    values: offset,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Total Repaid".to_string(),
                                                    color: "#d36cff",
                                                    values: total_repaid,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Total Repaid (Worst Case)".to_string(),
                                                    color: "#d36cff",
                                                    values: worst_total_repaid,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                            ]
                                        />
                                        <MultiLineChart
                                            title="Monthly Principal, Interest and Offset Top-Up".to_string()
                                            period_months=monthly_months
                                            lines=vec![
                                                ChartLine {
                                                    name: "Principal (Monthly)".to_string(),
                                                    color: "#25c59a",
                                                    values: monthly_principal,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Principal (Worst Case)".to_string(),
                                                    color: "#25c59a",
                                                    values: worst_monthly_principal,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                                ChartLine {
                                                    name: "Interest (Monthly)".to_string(),
                                                    color: "#5aa9ff",
                                                    values: monthly_interest,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Interest (Worst Case)".to_string(),
                                                    color: "#5aa9ff",
                                                    values: worst_monthly_interest,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                                ChartLine {
                                                    name: "Offset Top-Up (Monthly)".to_string(),
                                                    color: "#f59f42",
                                                    values: monthly_offset,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Offset Top-Up (Worst Case)".to_string(),
                                                    color: "#f59f42",
                                                    values: worst_monthly_offset,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                                ChartLine {
                                                    name: "Total Repayment (Monthly)".to_string(),
                                                    color: "#d36cff",
                                                    values: monthly_total_repayment,
                                                    opacity: 1.0,
                                                    dashed: false,
                                                },
                                                ChartLine {
                                                    name: "Total Repayment (Worst Case)".to_string(),
                                                    color: "#d36cff",
                                                    values: worst_monthly_total_repayment,
                                                    opacity: 0.35,
                                                    dashed: true,
                                                },
                                            ]
                                        />
                                    </section>
                                    <AmortizationTable rows=rows period_months=period_months />
                                </section>
                            }
                                .into_view()
                        }
                        Err(MortgageValidationError::Validation(issues)) => {
                            view! {
                                <section>
                                    <h3>"Validation issues"</h3>
                                    <ul>
                                        {issues
                                            .iter()
                                            .map(|i| {
                                                view! { <li class="error">{format!("{}: {}", i.field, i.message)}</li> }
                                            })
                                            .collect_view()}
                                    </ul>
                                </section>
                            }
                                .into_view()
                        }
                    }
                }}
            </div>
        </section>
    }
}
