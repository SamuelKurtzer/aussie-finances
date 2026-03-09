use leptos::*;

use crate::components::field_group::FieldGroup;
use crate::domain::types::{CalculatorInput, PayFrequency};

#[component]
pub fn CalculatorForm(input: RwSignal<CalculatorInput>) -> impl IntoView {
    let update_number = move |field: &'static str, value: String| {
        let parsed = value.parse::<f64>().unwrap_or(0.0);
        input.update(|state| match field {
            "gross_income_annual" => state.gross_income_annual = parsed,
            "super_rate_percent" => state.super_rate_percent = parsed,
            "deductions_annual" => state.deductions_annual = parsed,
            "salary_sacrifice_annual" => state.salary_sacrifice_annual = parsed,
            "reportable_fringe_benefits_annual" => state.reportable_fringe_benefits_annual = parsed,
            "mls_income_for_surcharge_annual" => {
                state.mls_income_for_surcharge_annual =
                    if parsed > 0.0 { Some(parsed) } else { None }
            }
            _ => {}
        });
    };

    view! {
        <div class="grid">
            <FieldGroup label="Income and Pay Cycle" help="Gross annual income is required. Choose how to display pay-period outputs.">
                <label for="gross-income">"Gross income (annual, AUD)"</label>
                <input
                    id="gross-income"
                    type="number"
                    min="0"
                    prop:value=move || input.get().gross_income_annual
                    on:input=move |ev| update_number("gross_income_annual", event_target_value(&ev))
                />

                <label for="pay-frequency">"Pay frequency"</label>
                <select
                    id="pay-frequency"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        input.update(|s| {
                            s.pay_frequency = match val.as_str() {
                                "weekly" => PayFrequency::Weekly,
                                "fortnightly" => PayFrequency::Fortnightly,
                                "monthly" => PayFrequency::Monthly,
                                _ => PayFrequency::Annually,
                            }
                        })
                    }
                >
                    <option value="weekly" selected=move || input.get().pay_frequency == PayFrequency::Weekly>"Weekly"</option>
                    <option value="fortnightly" selected=move || input.get().pay_frequency == PayFrequency::Fortnightly>"Fortnightly"</option>
                    <option value="monthly" selected=move || input.get().pay_frequency == PayFrequency::Monthly>"Monthly"</option>
                    <option value="annually" selected=move || input.get().pay_frequency == PayFrequency::Annually>"Annually"</option>
                </select>
            </FieldGroup>

            <FieldGroup label="Super and Student Debt" help="Control whether gross includes super and whether HELP/HECS repayment applies.">
                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().includes_super
                        on:change=move |ev| input.update(|s| s.includes_super = event_target_checked(&ev))
                    />
                    <span>"Gross figure includes super"</span>
                </label>

                <label for="super-rate">"Super rate (%)"</label>
                <input
                    id="super-rate"
                    type="number"
                    min="0"
                    max="25"
                    step="0.1"
                    prop:value=move || input.get().super_rate_percent
                    on:input=move |ev| update_number("super_rate_percent", event_target_value(&ev))
                />

                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().has_help_debt
                        on:change=move |ev| input.update(|s| s.has_help_debt = event_target_checked(&ev))
                    />
                    <span>"Has HELP/HECS debt"</span>
                </label>
            </FieldGroup>

            <FieldGroup label="Tax Adjustments" help="Enter annual deductions and salary sacrifice amounts.">
                <label for="deductions">"Deductions (annual, AUD)"</label>
                <input
                    id="deductions"
                    type="number"
                    min="0"
                    prop:value=move || input.get().deductions_annual
                    on:input=move |ev| update_number("deductions_annual", event_target_value(&ev))
                />

                <label for="sacrifice">"Salary sacrifice (annual, AUD)"</label>
                <input
                    id="sacrifice"
                    type="number"
                    min="0"
                    prop:value=move || input.get().salary_sacrifice_annual
                    on:input=move |ev| update_number("salary_sacrifice_annual", event_target_value(&ev))
                />
            </FieldGroup>

            <FieldGroup label="Medicare and Fringe Benefits" help="Use private health toggle and optional MLS income override when needed.">
                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().has_private_hospital_cover
                        on:change=move |ev| input.update(|s| s.has_private_hospital_cover = event_target_checked(&ev))
                    />
                    <span>"Has private hospital cover"</span>
                </label>

                <label for="rfb">"Reportable fringe benefits (annual, AUD)"</label>
                <input
                    id="rfb"
                    type="number"
                    min="0"
                    prop:value=move || input.get().reportable_fringe_benefits_annual
                    on:input=move |ev| update_number("reportable_fringe_benefits_annual", event_target_value(&ev))
                />

                <label for="mls-override">"MLS income override (annual, AUD, optional)"</label>
                <input
                    id="mls-override"
                    type="number"
                    min="0"
                    prop:value=move || input
                        .get()
                        .mls_income_for_surcharge_annual
                        .unwrap_or_default()
                    on:input=move |ev| update_number("mls_income_for_surcharge_annual", event_target_value(&ev))
                />
            </FieldGroup>
        </div>
    }
}
