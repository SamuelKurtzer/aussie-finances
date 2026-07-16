use leptos::*;

use crate::components::split_editor::SplitEditor;
use crate::domain::mortgages::{MortgagePortfolioInput, SplitInput, MAX_SPLITS_PER_MORTGAGE};

#[component]
pub fn MortgageEditor(
    portfolio: RwSignal<MortgagePortfolioInput>,
    mortgage_id: u32,
) -> impl IntoView {
    view! {
        <section class="field-group">
            <div class="row-head">
                <h3>
                    {move || {
                        portfolio
                            .get()
                            .mortgages
                            .iter()
                            .find(|m| m.id == mortgage_id)
                            .map(|m| m.name.clone())
                            .unwrap_or_default()
                    }}
                </h3>
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
                        if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id) {
                            m.name = value.clone();
                        }
                    });
                }
            />

            <div class="three-col">
                <div>
                    <label>"Home value (AUD)"</label>
                    <input
                        type="number" inputmode="decimal"
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
                                if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id) {
                                    m.home_value = value;
                                }
                            });
                        }
                    />
                </div>
                <div>
                    <label>"Offset balance (AUD)"</label>
                    <input
                        type="number" inputmode="decimal"
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
                                if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id) {
                                    m.offset_balance = value;
                                }
                            });
                        }
                    />
                </div>
                <div>
                    <label>"Loan length (months)"</label>
                    <input
                        type="number" inputmode="decimal"
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
                                if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id) {
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
                            if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id) {
                                if m.splits.len() >= MAX_SPLITS_PER_MORTGAGE {
                                    return;
                                }
                                let id = m.next_split_id();
                                m.splits.push(SplitInput {
                                    id,
                                    name: format!("Split {}", id),
                                    ..Default::default()
                                });
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
                    view! {
                        <SplitEditor portfolio=portfolio mortgage_id=mortgage_id split_id=split.id />
                    }
                }
            />
        </section>
    }
}
