use leptos::*;

use crate::domain::mortgages::{
    LoanPurpose, LoanRepaymentType, MortgagePortfolioInput, RateType,
};

#[component]
pub fn SplitEditor(
    portfolio: RwSignal<MortgagePortfolioInput>,
    mortgage_id: u32,
    split_id: u32,
) -> impl IntoView {
    view! {
        <div class="split-block">
            <div class="row-head">
                <h4>
                    {move || {
                        portfolio
                            .get()
                            .mortgages
                            .iter()
                            .find(|m| m.id == mortgage_id)
                            .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
                            .map(|s| s.name.clone())
                            .unwrap_or_default()
                    }}
                </h4>
                <button
                    type="button"
                    class="secondary"
                    on:click=move |_| {
                        portfolio.update(|p| {
                            if let Some(m) = p.mortgages.iter_mut().find(|m| m.id == mortgage_id) {
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
                                .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                        type="number" inputmode="decimal"
                        min="0"
                        prop:value=move || {
                            portfolio
                                .get()
                                .mortgages
                                .iter()
                                .find(|m| m.id == mortgage_id)
                                .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                        type="number" inputmode="decimal"
                        min="0"
                        step="0.01"
                        prop:value=move || {
                            portfolio
                                .get()
                                .mortgages
                                .iter()
                                .find(|m| m.id == mortgage_id)
                                .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                        type="number" inputmode="decimal"
                        min="0"
                        step="0.5"
                        prop:value=move || {
                            portfolio
                                .get()
                                .mortgages
                                .iter()
                                .find(|m| m.id == mortgage_id)
                                .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter_mut().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
                                    .and_then(|m| m.splits.iter().find(|s| s.id == split_id))
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
