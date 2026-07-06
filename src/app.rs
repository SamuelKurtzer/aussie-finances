use leptos::*;
use leptos_router::*;

use crate::pages::debt_recycling::DebtRecyclingPage;
use crate::pages::income::IncomePage;
use crate::pages::mortgages::MortgagesPage;
use crate::pages::spreadsheet::SpreadsheetPage;
use crate::pages::tax::TaxPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="site">
                <header class="hero">
                    <h1>"aus-fin " <span class="badge">"v0.1"</span></h1>
                    <p class="muted">
                        "Income tax, mortgage amortization, and debt recycling projections for Australians. All data stays in your browser."
                    </p>
                </header>
                <section class="workspace">
                    <nav class="tabs">
                        <A href="/income">"Income Calculator"</A>
                        <A href="/tax">"Tax"</A>
                        <A href="/mortgages">"Mortgages"</A>
                        <A href="/debt-recycling">"Debt Recycling"</A>
                        <A href="/spreadsheet">"Spreadsheet"</A>
                    </nav>
                    <div class="workspace-body">
                        <Routes>
                            <Route path="" view=|| view! { <IncomePage /> } />
                            <Route path="/income" view=|| view! { <IncomePage /> } />
                            <Route path="/tax" view=|| view! { <TaxPage /> } />
                            <Route path="/mortgages" view=|| view! { <MortgagesPage /> } />
                            <Route path="/debt-recycling" view=|| view! { <DebtRecyclingPage /> } />
                            <Route path="/spreadsheet" view=|| view! { <SpreadsheetPage /> } />
                        </Routes>
                    </div>
                </section>
            </main>
        </Router>
    }
}
