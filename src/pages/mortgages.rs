use leptos::*;

use crate::components::collapsible::Collapsible;
use crate::components::mortgage_editor::MortgageEditor;
use crate::components::mortgage_results::MortgageResults;
use crate::domain::mortgages::{
    calculate_mortgage_portfolio, DebtRecycleInput, MortgageInput, MortgagePortfolioInput,
    RepaymentCadence, MAX_MORTGAGES,
};
use crate::storage::{
    load_from_storage, load_raw_from_storage, remove_from_storage, save_to_storage,
    DEBT_RECYCLE_STORAGE_KEY, INCOME_STORAGE_KEY, MORTGAGE_STORAGE_KEY,
};

#[component]
pub fn MortgagesPage() -> impl IntoView {
    let portfolio = create_rw_signal(
        load_from_storage::<MortgagePortfolioInput>(MORTGAGE_STORAGE_KEY).unwrap_or_default(),
    );

    create_effect(move |_| {
        save_to_storage(MORTGAGE_STORAGE_KEY, &portfolio.get());
    });

    let income_context = create_memo(move |_| {
        load_raw_from_storage(INCOME_STORAGE_KEY)
            .as_deref()
            .and_then(crate::domain::mortgages::load_income_context_from_saved_input)
    });
    let result = create_memo(move |_| {
        let income = income_context.get();
        let mut input = portfolio.get();
        input.debt_recycle = load_from_storage::<DebtRecycleInput>(DEBT_RECYCLE_STORAGE_KEY);
        calculate_mortgage_portfolio(&input, income.as_ref())
    });

    let add_mortgage = {
        let portfolio = portfolio;
        move |_| {
            portfolio.update(|p| {
                if p.mortgages.len() >= MAX_MORTGAGES {
                    return;
                }
                let id = p.next_mortgage_id();
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
            remove_from_storage(MORTGAGE_STORAGE_KEY);
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

                <Collapsible title="Portfolio Settings" class="field-group">
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
                        type="number" inputmode="decimal"
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
                </Collapsible>

                <For
                    each=move || portfolio.get().mortgages
                    key=|m| m.id
                    children=move |mortgage| {
                        view! {
                            <MortgageEditor portfolio=portfolio mortgage_id=mortgage.id />
                        }
                    }
                />
            </div>

            <div>
                {move || {
                    view! { <MortgageResults result=result.get() /> }
                }}
            </div>
        </section>
    }
}
