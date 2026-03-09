use leptos::*;
use leptos_router::*;

use crate::pages::debt_recycling::DebtRecyclingPage;
use crate::pages::income::IncomePage;
use crate::pages::mortgages::MortgagesPage;
use crate::pages::tax::TaxPage;

#[component]
pub fn App() -> impl IntoView {
    view! {
        <Router>
            <main class="site">
                <header class="hero">
                    <span class="badge">"MVP"</span>
                    <h1>"Aus Fin: Practical Money Tools for Australians"</h1>
                    <p class="muted">
                        "Start with your take-home pay estimate, then expand into tax, mortgages, and debt recycling strategy."
                    </p>
                </header>
                <section class="workspace">
                    <nav class="tabs">
                        <A href="/income">"Income Calculator"</A>
                        <A href="/tax">"Tax"</A>
                        <A href="/mortgages">"Mortgages"</A>
                        <A href="/debt-recycling">"Debt Recycling"</A>
                    </nav>
                    <div class="workspace-body">
                        <Routes>
                            <Route path="" view=|| view! { <IncomePage /> } />
                            <Route path="/income" view=|| view! { <IncomePage /> } />
                            <Route path="/tax" view=|| view! { <TaxPage /> } />
                            <Route path="/mortgages" view=|| view! { <MortgagesPage /> } />
                            <Route path="/debt-recycling" view=|| view! { <DebtRecyclingPage /> } />
                        </Routes>
                    </div>
                </section>
            </main>
        </Router>
    }
}
