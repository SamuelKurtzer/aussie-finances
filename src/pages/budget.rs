use leptos::*;

use crate::components::collapsible::Collapsible;
use crate::domain::budget::{BudgetInput, ExpenseFrequency, ExpenseItem};
use crate::formatting::fmt_money;
use crate::pages::income::load_household_outgoings;
use crate::storage::{
    load_from_storage, load_raw_from_storage, save_to_storage, BUDGET_STORAGE_KEY,
    INCOME_STORAGE_KEY,
};

fn load_monthly_net_income() -> Option<f64> {
    let raw = load_raw_from_storage(INCOME_STORAGE_KEY)?;
    let input = serde_json::from_str::<crate::domain::types::CalculatorInput>(&raw).ok()?;
    let rules = crate::domain::tax_rules::TaxRules::for_year(input.financial_year);
    let output = crate::domain::calculator::calculate_income(&input, &rules).ok()?;
    Some(output.net_income_annual / 12.0)
}

#[component]
pub fn BudgetPage() -> impl IntoView {
    let budget = create_rw_signal(
        load_from_storage::<BudgetInput>(BUDGET_STORAGE_KEY).unwrap_or_default(),
    );

    create_effect(move |_| {
        save_to_storage(BUDGET_STORAGE_KEY, &budget.get());
    });

    let add_item = move |_| {
        budget.update(|b| {
            let id = b.next_id();
            b.items.push(ExpenseItem {
                id,
                name: String::new(),
                amount: 0.0,
                frequency: ExpenseFrequency::Monthly,
            });
        });
    };

    let monthly_net = load_monthly_net_income();
    let (mortgage_annual, dr_annual) = load_household_outgoings();
    let other_outgoings_monthly = (mortgage_annual + dr_annual) / 12.0;

    view! {
        <section>
            <h2>"Budget"</h2>
            <p class="muted">
                "Track ongoing expenses. Amounts are converted to monthly equivalents and compared against your net income from the Income Calculator tab."
            </p>

            <Collapsible title="Ongoing Expenses" class="field-group">
                {
                    // Memo so typing in a row doesn't rebuild the table and steal focus.
                    let has_items = create_memo(move |_| !budget.get().items.is_empty());
                    move || has_items.get().then(|| view! {
                        <div class="table-wrap">
                            <table class="budget-table">
                                <thead>
                                    <tr>
                                        <th>"Expense"</th>
                                        <th>"Amount (AUD)"</th>
                                        <th>"Frequency"</th>
                                        <th>"Monthly"</th>
                                        <th>"Annual"</th>
                                        <th></th>
                                    </tr>
                                </thead>
                                <tbody>
                                    <For
                                        each=move || budget.get().items.clone()
                                        key=|item| item.id
                                        children=move |item| {
                                            let id = item.id;
                                            let current = move || {
                                                budget
                                                    .get()
                                                    .items
                                                    .iter()
                                                    .find(|i| i.id == id)
                                                    .cloned()
                                                    .unwrap_or(ExpenseItem {
                                                        id,
                                                        name: String::new(),
                                                        amount: 0.0,
                                                        frequency: ExpenseFrequency::Monthly,
                                                    })
                                            };
                                            view! {
                                                <tr>
                                                    <td>
                                                        <input
                                                            type="text"
                                                            placeholder="e.g. Groceries"
                                                            prop:value=move || current().name
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev);
                                                                budget.update(|b| {
                                                                    if let Some(i) = b.items.iter_mut().find(|i| i.id == id) {
                                                                        i.name = value.clone();
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </td>
                                                    <td>
                                                        <input
                                                            type="number" inputmode="decimal"
                                                            min="0"
                                                            step="0.01"
                                                            prop:value=move || current().amount
                                                            on:input=move |ev| {
                                                                let value = event_target_value(&ev)
                                                                    .parse::<f64>()
                                                                    .unwrap_or(0.0)
                                                                    .max(0.0);
                                                                budget.update(|b| {
                                                                    if let Some(i) = b.items.iter_mut().find(|i| i.id == id) {
                                                                        i.amount = value;
                                                                    }
                                                                });
                                                            }
                                                        />
                                                    </td>
                                                    <td>
                                                        <select
                                                            on:change=move |ev| {
                                                                let freq = ExpenseFrequency::from_key(&event_target_value(&ev));
                                                                budget.update(|b| {
                                                                    if let Some(i) = b.items.iter_mut().find(|i| i.id == id) {
                                                                        i.frequency = freq;
                                                                    }
                                                                });
                                                            }
                                                        >
                                                            {ExpenseFrequency::ALL
                                                                .into_iter()
                                                                .map(|freq| {
                                                                    view! {
                                                                        <option
                                                                            value={freq.key()}
                                                                            selected=move || current().frequency == freq
                                                                        >
                                                                            {freq.label()}
                                                                        </option>
                                                                    }
                                                                })
                                                                .collect_view()}
                                                        </select>
                                                    </td>
                                                    <td>{move || fmt_money(current().monthly_amount())}</td>
                                                    <td>{move || fmt_money(current().annual_amount())}</td>
                                                    <td>
                                                        <button
                                                            type="button"
                                                            class="secondary"
                                                            on:click=move |_| {
                                                                budget.update(|b| b.items.retain(|i| i.id != id));
                                                            }
                                                        >
                                                            "Remove"
                                                        </button>
                                                    </td>
                                                </tr>
                                            }
                                        }
                                    />
                                </tbody>
                            </table>
                        </div>
                    })
                }
                <button type="button" on:click=add_item>"+ Add Expense"</button>
            </Collapsible>

            <Collapsible title="Budget Summary">
                <div class="summary-grid">
                    <article class="mini-card">
                        <span class="muted">"Monthly Expenses"</span>
                        <strong>{move || fmt_money(budget.get().monthly_total())}</strong>
                    </article>
                    <article class="mini-card">
                        <span class="muted">"Annual Expenses"</span>
                        <strong>{move || fmt_money(budget.get().annual_total())}</strong>
                    </article>
                    {monthly_net.map(|net| view! {
                        <article class="mini-card">
                            <span class="muted">"Monthly Net Income"</span>
                            <strong>{fmt_money(net)}</strong>
                        </article>
                        {(other_outgoings_monthly > 0.005).then(|| view! {
                            <article class="mini-card">
                                <span class="muted">"Mortgage + Debt Recycling (Monthly)"</span>
                                <strong>{fmt_money(other_outgoings_monthly)}</strong>
                            </article>
                        })}
                        <article class="mini-card">
                            <span class="muted">"Left Over Each Month"</span>
                            <strong class=move || {
                                if net - other_outgoings_monthly - budget.get().monthly_total() < 0.0 {
                                    "error"
                                } else {
                                    ""
                                }
                            }>
                                {move || fmt_money(net - other_outgoings_monthly - budget.get().monthly_total())}
                            </strong>
                        </article>
                    })}
                </div>
                {monthly_net.is_none().then(|| view! {
                    <p class="muted">
                        "Set up the Income Calculator tab to compare expenses against your net income."
                    </p>
                })}
            </Collapsible>
        </section>
    }
}
