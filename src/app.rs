use leptos::*;
use serde::{Deserialize, Serialize};

use crate::pages::budget::BudgetPage;
use crate::pages::debt_recycling::DebtRecyclingPage;
use crate::pages::income::IncomePage;
use crate::pages::mortgages::MortgagesPage;
use crate::pages::spreadsheet::SpreadsheetPage;
use crate::storage::{load_from_storage, save_to_storage, ACTIVE_TAB_STORAGE_KEY};

#[derive(Clone, Copy, Debug, PartialEq, Eq, Serialize, Deserialize, Default)]
pub enum Tab {
    #[default]
    Income,
    Mortgages,
    DebtRecycling,
    Budget,
    Spreadsheet,
}

impl Tab {
    const ALL: [Tab; 5] = [
        Tab::Income,
        Tab::Mortgages,
        Tab::DebtRecycling,
        Tab::Budget,
        Tab::Spreadsheet,
    ];

    fn label(self) -> &'static str {
        match self {
            Tab::Income => "Income Calculator",
            Tab::Mortgages => "Mortgages",
            Tab::DebtRecycling => "Debt Recycling",
            Tab::Budget => "Budget",
            Tab::Spreadsheet => "Spreadsheet",
        }
    }
}

#[component]
pub fn App() -> impl IntoView {
    let active_tab =
        create_rw_signal(load_from_storage::<Tab>(ACTIVE_TAB_STORAGE_KEY).unwrap_or_default());

    create_effect(move |_| {
        save_to_storage(ACTIVE_TAB_STORAGE_KEY, &active_tab.get());
    });

    view! {
        <main class="site">
            <header class="hero">
                <h1>"aus-fin " <span class="badge">"v0.1"</span></h1>
                <p class="muted">
                    "Income tax, mortgage amortization, and debt recycling projections for Australians. All data stays in your browser."
                </p>
            </header>
            <section class="workspace">
                <nav class="tabs">
                    {Tab::ALL
                        .into_iter()
                        .map(|tab| {
                            view! {
                                <button
                                    type="button"
                                    attr:aria-current=move || (active_tab.get() == tab).then_some("page")
                                    on:click=move |_| active_tab.set(tab)
                                >
                                    {tab.label()}
                                </button>
                            }
                        })
                        .collect_view()}
                </nav>
                <div class="workspace-body">
                    {move || match active_tab.get() {
                        Tab::Income => view! { <IncomePage /> },
                        Tab::Mortgages => view! { <MortgagesPage /> },
                        Tab::DebtRecycling => view! { <DebtRecyclingPage /> },
                        Tab::Budget => view! { <BudgetPage /> },
                        Tab::Spreadsheet => view! { <SpreadsheetPage /> },
                    }}
                </div>
            </section>
        </main>
    }
}
