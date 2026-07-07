use leptos::*;

use crate::components::field_group::FieldGroup;
use crate::components::info_tip::InfoTip;
use crate::domain::types::{
    CalculatorInput, ContributionFrequency, FinancialYear, IncomeUnit, MedicareExemption,
    PayFrequency, Residency,
};

#[component]
pub fn CalculatorForm(input: RwSignal<CalculatorInput>) -> impl IntoView {
    let update_number = move |field: &'static str, value: String| {
        let parsed = value.parse::<f64>().unwrap_or(0.0);
        input.update(|state| match field {
            "income_amount" => state.income_amount = parsed,
            "hours_per_week" => state.hours_per_week = parsed,
            "days_per_week" => state.days_per_week = parsed,
            "bonus_annual" => state.bonus_annual = parsed,
            "overtime_annual" => state.overtime_annual = parsed,
            "super_rate_percent" => state.super_rate_percent = parsed,
            "deductions_annual" => state.deductions_annual = parsed,
            "salary_sacrifice_amount" => state.salary_sacrifice_amount = parsed,
            "extra_super_annual" => state.extra_super_annual = parsed,
            "reportable_fringe_benefits_annual" => state.reportable_fringe_benefits_annual = parsed,
            "dependants" => state.dependants = parsed.max(0.0) as u32,
            "family_income_annual" => {
                state.family_income_annual = if parsed > 0.0 { Some(parsed) } else { None }
            }
            "mls_income_for_surcharge_annual" => {
                state.mls_income_for_surcharge_annual =
                    if parsed > 0.0 { Some(parsed) } else { None }
            }
            _ => {}
        });
    };

    view! {
        <div class="grid">
            <FieldGroup label="Income and Pay Cycle" help="Pick the financial year, enter income in any unit, and choose how to display pay-period outputs.">
                <label for="financial-year">"Financial year"</label>
                <select
                    id="financial-year"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        input.update(|s| {
                            s.financial_year = match val.as_str() {
                                "fy2024_25" => FinancialYear::Fy2024_25,
                                _ => FinancialYear::Fy2025_26,
                            }
                        })
                    }
                >
                    <option value="fy2025_26" selected=move || input.get().financial_year == FinancialYear::Fy2025_26>"2025-26"</option>
                    <option value="fy2024_25" selected=move || input.get().financial_year == FinancialYear::Fy2024_25>"2024-25"</option>
                </select>

                <label for="income-amount">"Income amount (AUD)"</label>
                <input
                    id="income-amount"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input.get().income_amount
                    on:input=move |ev| update_number("income_amount", event_target_value(&ev))
                />

                <label for="income-unit">"Income unit"</label>
                <select
                    id="income-unit"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        input.update(|s| {
                            s.income_unit = match val.as_str() {
                                "hourly" => IncomeUnit::Hourly,
                                "daily" => IncomeUnit::Daily,
                                "weekly" => IncomeUnit::Weekly,
                                "fortnightly" => IncomeUnit::Fortnightly,
                                "monthly" => IncomeUnit::Monthly,
                                _ => IncomeUnit::Annual,
                            }
                        })
                    }
                >
                    <option value="annual" selected=move || input.get().income_unit == IncomeUnit::Annual>"Per year"</option>
                    <option value="monthly" selected=move || input.get().income_unit == IncomeUnit::Monthly>"Per month"</option>
                    <option value="fortnightly" selected=move || input.get().income_unit == IncomeUnit::Fortnightly>"Per fortnight"</option>
                    <option value="weekly" selected=move || input.get().income_unit == IncomeUnit::Weekly>"Per week"</option>
                    <option value="daily" selected=move || input.get().income_unit == IncomeUnit::Daily>"Per day"</option>
                    <option value="hourly" selected=move || input.get().income_unit == IncomeUnit::Hourly>"Per hour"</option>
                </select>

                {move || (input.get().income_unit == IncomeUnit::Hourly).then(|| view! {
                    <label for="hours-per-week">"Hours per week"</label>
                    <input
                        id="hours-per-week"
                        type="number"
                        min="1"
                        max="100"
                        step="0.5"
                        prop:value=move || input.get().hours_per_week
                        on:input=move |ev| update_number("hours_per_week", event_target_value(&ev))
                    />
                })}

                {move || (input.get().income_unit == IncomeUnit::Daily).then(|| view! {
                    <label for="days-per-week">"Days per week"</label>
                    <input
                        id="days-per-week"
                        type="number"
                        min="1"
                        max="7"
                        step="0.5"
                        prop:value=move || input.get().days_per_week
                        on:input=move |ev| update_number("days_per_week", event_target_value(&ev))
                    />
                })}

                <label for="bonus">"Bonus (annual, AUD)"</label>
                <input
                    id="bonus"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input.get().bonus_annual
                    on:input=move |ev| update_number("bonus_annual", event_target_value(&ev))
                />

                <label for="overtime">"Overtime (annual, AUD)"</label>
                <input
                    id="overtime"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input.get().overtime_annual
                    on:input=move |ev| update_number("overtime_annual", event_target_value(&ev))
                />

                <label for="pay-frequency">
                    "Pay frequency (display)"
                    <InfoTip text="Only changes how results are displayed per pay period. It does not affect the calculation." />
                </label>
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

                <label for="residency">
                    "Residency status"
                    <InfoTip text="Residency for tax purposes, which can differ from visa status. Non-residents pay no Medicare levy but get no tax-free threshold; Medicare and offset options are hidden for them." />
                </label>
                <select
                    id="residency"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        input.update(|s| {
                            s.residency = match val.as_str() {
                                "non_resident" => Residency::NonResident,
                                "whm" => Residency::WorkingHolidayMaker,
                                _ => Residency::Resident,
                            }
                        })
                    }
                >
                    <option value="resident" selected=move || input.get().residency == Residency::Resident>"Australian resident"</option>
                    <option value="non_resident" selected=move || input.get().residency == Residency::NonResident>"Non-resident"</option>
                    <option value="whm" selected=move || input.get().residency == Residency::WorkingHolidayMaker>"Working holiday maker"</option>
                </select>
            </FieldGroup>

            <FieldGroup label="Super and Student Debt" help="Control whether the salary includes super and whether study loan repayments apply.">
                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().includes_super
                        on:change=move |ev| input.update(|s| s.includes_super = event_target_checked(&ev))
                    />
                    <span>"Salary figure includes super"</span>
                    <InfoTip text="Tick if the salary you entered is a package that already includes employer super. It will be converted to a super-exclusive base." />
                </label>

                <label for="super-rate">
                    "Super rate (%)"
                    <InfoTip text="The superannuation guarantee rate your employer pays. The legal minimum is 12% from 1 July 2025 (11.5% in 2024-25)." />
                </label>
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
                        prop:checked=move || input.get().maximize_super
                        on:change=move |ev| input.update(|s| s.maximize_super = event_target_checked(&ev))
                    />
                    <span>"Max out concessional super cap"</span>
                    <InfoTip text="Automatically tops up extra before-tax super so employer super plus salary sacrifice plus extra contributions reach the $30,000 concessional cap." />
                </label>

                {move || (!input.get().maximize_super).then(|| view! {
                    <label for="extra-super">
                        "Extra super (concessional, annual, AUD)"
                        <InfoTip text="Personal before-tax contributions on top of employer super. They reduce taxable income and count toward the $30,000 concessional cap." />
                    </label>
                    <input
                        id="extra-super"
                        type="number"
                        min="0"
                        step="0.01"
                        prop:value=move || input.get().extra_super_annual
                        on:input=move |ev| update_number("extra_super_annual", event_target_value(&ev))
                    />
                })}

                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().has_help_debt
                        on:change=move |ev| input.update(|s| s.has_help_debt = event_target_checked(&ev))
                    />
                    <span>"Has study loan (HELP, VET, SSL, TSL, SFSS)"</span>
                    <InfoTip text="Compulsory repayments are withheld once your repayment income passes the year's threshold. All study loan types use the same repayment rates." />
                </label>
            </FieldGroup>

            <FieldGroup label="Tax Adjustments" help="Enter annual deductions and salary sacrifice amounts.">
                <label for="deductions">
                    "Deductions (annual, AUD)"
                    <InfoTip text="Work-related expenses and other allowable deductions. They reduce your taxable income, not your tax directly." />
                </label>
                <input
                    id="deductions"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input.get().deductions_annual
                    on:input=move |ev| update_number("deductions_annual", event_target_value(&ev))
                />

                <label for="sacrifice">
                    "Salary sacrifice (AUD)"
                    <InfoTip text="Pre-tax salary directed into super. Reduces taxable income and take-home pay; counts toward the concessional cap. Enter per year or per month." />
                </label>
                <input
                    id="sacrifice"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input.get().salary_sacrifice_amount
                    on:input=move |ev| update_number("salary_sacrifice_amount", event_target_value(&ev))
                />

                <label for="sacrifice-frequency">"Salary sacrifice frequency"</label>
                <select
                    id="sacrifice-frequency"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        input.update(|s| {
                            s.salary_sacrifice_frequency = match val.as_str() {
                                "monthly" => ContributionFrequency::Monthly,
                                _ => ContributionFrequency::Annual,
                            }
                        })
                    }
                >
                    <option value="annual" selected=move || input.get().salary_sacrifice_frequency == ContributionFrequency::Annual>"Per year"</option>
                    <option value="monthly" selected=move || input.get().salary_sacrifice_frequency == ContributionFrequency::Monthly>"Per month"</option>
                </select>
            </FieldGroup>

            {move || (input.get().residency == Residency::Resident).then(|| view! {
            <FieldGroup label="Medicare, Family, and Offsets" help="Exemptions, family thresholds, seniors offset, private cover, and fringe benefits.">
                <label for="medicare-exemption">
                    "Medicare exemption"
                    <InfoTip text="Full or half levy exemption for certain groups, e.g. some foreign residents, defence force members, or blindness/sight pension recipients. Most people have no exemption." />
                </label>
                <select
                    id="medicare-exemption"
                    on:change=move |ev| {
                        let val = event_target_value(&ev);
                        input.update(|s| {
                            s.medicare_exemption = match val.as_str() {
                                "half" => MedicareExemption::Half,
                                "full" => MedicareExemption::Full,
                                _ => MedicareExemption::None,
                            }
                        })
                    }
                >
                    <option value="none" selected=move || input.get().medicare_exemption == MedicareExemption::None>"None"</option>
                    <option value="half" selected=move || input.get().medicare_exemption == MedicareExemption::Half>"Half exemption"</option>
                    <option value="full" selected=move || input.get().medicare_exemption == MedicareExemption::Full>"Full exemption"</option>
                </select>

                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().is_sapto_eligible
                        on:change=move |ev| input.update(|s| s.is_sapto_eligible = event_target_checked(&ev))
                    />
                    <span>"Senior or pensioner (SAPTO)"</span>
                    <InfoTip text="Seniors and Pensioners Tax Offset: reduces income tax by up to $2,230 if you are Age Pension age and under the income limit." />
                </label>

                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().has_family
                        on:change=move |ev| input.update(|s| s.has_family = event_target_checked(&ev))
                    />
                    <span>"Family (spouse or dependants)"</span>
                    <InfoTip text="Having a spouse or dependent children raises the Medicare levy low-income threshold and switches the surcharge to family income tiers." />
                </label>

                {move || (input.get().has_family || input.get().dependants > 0).then(|| view! {
                    <label for="dependants">"Dependent children"</label>
                    <input
                        id="dependants"
                        type="number"
                        min="0"
                        step="1"
                        prop:value=move || input.get().dependants
                        on:input=move |ev| update_number("dependants", event_target_value(&ev))
                    />

                    <label for="family-income">"Combined family income (annual, AUD, optional)"</label>
                    <input
                        id="family-income"
                        type="number"
                        min="0"
                        step="0.01"
                        prop:value=move || input.get().family_income_annual.unwrap_or_default()
                        on:input=move |ev| update_number("family_income_annual", event_target_value(&ev))
                    />
                })}

                <label class="check-row">
                    <input
                        type="checkbox"
                        prop:checked=move || input.get().has_private_hospital_cover
                        on:change=move |ev| input.update(|s| s.has_private_hospital_cover = event_target_checked(&ev))
                    />
                    <span>"Has private hospital cover"</span>
                    <InfoTip text="An appropriate level of private hospital cover (not extras-only) exempts you from the Medicare Levy Surcharge." />
                </label>

                <label for="rfb">
                    "Reportable fringe benefits (annual, AUD)"
                    <InfoTip text="The grossed-up value of fringe benefits shown on your payment summary. Counts toward surcharge income but is not taxed as salary." />
                </label>
                <input
                    id="rfb"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input.get().reportable_fringe_benefits_annual
                    on:input=move |ev| update_number("reportable_fringe_benefits_annual", event_target_value(&ev))
                />

                <label for="mls-override">
                    "MLS income override (annual, AUD, optional)"
                    <InfoTip text="Income for surcharge purposes includes taxable income, fringe benefits, super contributions, and investment losses. Set it here directly if the default estimate is off." />
                </label>
                <input
                    id="mls-override"
                    type="number"
                    min="0"
                    step="0.01"
                    prop:value=move || input
                        .get()
                        .mls_income_for_surcharge_annual
                        .unwrap_or_default()
                    on:input=move |ev| update_number("mls_income_for_surcharge_annual", event_target_value(&ev))
                />
            </FieldGroup>
            })}
        </div>
    }
}
