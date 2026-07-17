use leptos::*;
use serde::{Deserialize, Serialize};

use crate::backup::{export_backup_json, trigger_download};
use crate::pages::budget::BudgetPage;
use crate::pages::debt_recycling::DebtRecyclingPage;
use crate::pages::income::IncomePage;
use crate::pages::mortgages::MortgagesPage;
use crate::pages::spreadsheet::SpreadsheetPage;
use crate::storage::{persisted_signal, ACTIVE_TAB_STORAGE_KEY};

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

    /// Compact labels so all five tabs fit one row on phone widths.
    fn short_label(self) -> &'static str {
        match self {
            Tab::Income => "Income",
            Tab::Mortgages => "Loans",
            Tab::DebtRecycling => "Recycle",
            Tab::Budget => "Budget",
            Tab::Spreadsheet => "Sheet",
        }
    }
}

fn export_backup() {
    let date = js_sys::Date::new_0().to_iso_string().as_string();
    let date = date.as_deref().map(|d| &d[..10]).unwrap_or("backup");
    trigger_download(
        &export_backup_json(),
        &format!("aus-fin-backup-{date}.json"),
        "application/json",
    );
}

#[cfg(target_arch = "wasm32")]
fn import_backup_from_input(ev: &leptos::ev::Event) {
    use wasm_bindgen::JsCast;

    let Some(input) = ev
        .target()
        .and_then(|t| t.dyn_into::<web_sys::HtmlInputElement>().ok())
    else {
        return;
    };
    let Some(file) = input.files().and_then(|files| files.get(0)) else {
        return;
    };
    input.set_value("");

    let Ok(reader) = web_sys::FileReader::new() else {
        return;
    };
    let reader_for_load = reader.clone();
    let onload = wasm_bindgen::closure::Closure::once(move |_ev: web_sys::ProgressEvent| {
        let Some(window) = web_sys::window() else {
            return;
        };
        let Some(text) = reader_for_load.result().ok().and_then(|v| v.as_string()) else {
            let _ = window.alert_with_message("Import failed: could not read file.");
            return;
        };
        let confirmed = window
            .confirm_with_message(
                "Importing replaces all aus-fin data on this device with the backup. Continue?",
            )
            .unwrap_or(false);
        if !confirmed {
            return;
        }
        match crate::backup::apply_backup(&text) {
            Ok(_) => {
                let _ = window.location().reload();
            }
            Err(message) => {
                let _ = window.alert_with_message(&format!("Import failed: {message}."));
            }
        }
    });
    reader.set_onload(Some(onload.as_ref().unchecked_ref()));
    onload.forget();
    let _ = reader.read_as_text(&file);
}

#[cfg(not(target_arch = "wasm32"))]
fn import_backup_from_input(_ev: &leptos::ev::Event) {}

#[component]
pub fn App() -> impl IntoView {
    let active_tab = persisted_signal::<Tab>(ACTIVE_TAB_STORAGE_KEY);

    // Keep the active tab visible when the tab row scrolls horizontally.
    create_effect(move |_| {
        active_tab.track();
        #[cfg(target_arch = "wasm32")]
        if let Some(button) = web_sys::window().and_then(|w| w.document()).and_then(|d| {
            d.query_selector(".tabs button[aria-current='page']")
                .ok()
                .flatten()
        }) {
            let options = web_sys::ScrollIntoViewOptions::new();
            options.set_block(web_sys::ScrollLogicalPosition::Nearest);
            options.set_inline(web_sys::ScrollLogicalPosition::Nearest);
            button.scroll_into_view_with_scroll_into_view_options(&options);
        }
    });

    let import_input: NodeRef<html::Input> = create_node_ref();

    view! {
        <main class="site">
            <header class="hero">
                <div class="hero-top">
                    <h1>"aus-fin " <span class="badge">{concat!("v", env!("CARGO_PKG_VERSION"))}</span></h1>
                    <div class="hero-actions">
                        <button type="button" class="secondary" on:click=move |_| export_backup()>
                            "Export Data"
                        </button>
                        <button
                            type="button"
                            class="secondary"
                            on:click=move |_| {
                                if let Some(input) = import_input.get() {
                                    input.click();
                                }
                            }
                        >
                            "Import Data"
                        </button>
                        <input
                            type="file"
                            accept="application/json,.json"
                            style="display:none"
                            node_ref=import_input
                            on:change=move |ev| import_backup_from_input(&ev)
                        />
                    </div>
                </div>
                <p class="muted">
                    "Income tax, mortgage amortization, and debt recycling projections for Australians. All data stays on your device."
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
                                    <span class="tab-label-full">{tab.label()}</span>
                                    <span class="tab-label-short">{tab.short_label()}</span>
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
