use leptos::*;

use crate::components::collapsible::Collapsible;
use crate::domain::types::CalculatorOutput;
use crate::formatting::{fmt_currency, fmt_money};

const TAU: f64 = std::f64::consts::TAU;

struct Slice {
    name: &'static str,
    color: &'static str,
    value: f64,
}

#[component]
pub fn TaxPieChart(
    result: CalculatorOutput,
    #[prop(default = 0.0)] mortgage_annual: f64,
    #[prop(default = 0.0)] debt_recycling_annual: f64,
    #[prop(default = 0.0)] expenses_annual: f64,
) -> impl IntoView {
    let gross = result.gross_income_annual;
    let income_tax_net =
        (result.income_tax_annual - result.lito_annual - result.sapto_annual).max(0.0);
    let medicare = result.medicare_levy_annual + result.medicare_levy_surcharge_annual;
    let sacrificed = (gross - result.net_income_annual - result.total_withheld_annual).max(0.0);
    let has_outgoings =
        mortgage_annual > 0.005 || debt_recycling_annual > 0.005 || expenses_annual > 0.005;
    let remaining_net =
        (result.net_income_annual - mortgage_annual - debt_recycling_annual - expenses_annual)
            .max(0.0);

    let slices: Vec<Slice> = [
        Slice {
            name: if has_outgoings {
                "Net income (remaining)"
            } else {
                "Net income"
            },
            color: "#98c379",
            value: remaining_net,
        },
        Slice {
            name: "Mortgage repayments",
            color: "#ffb000",
            value: mortgage_annual,
        },
        Slice {
            name: "Debt recycling",
            color: "#56b6c2",
            value: debt_recycling_annual,
        },
        Slice {
            name: "Budget expenses",
            color: "#d19a66",
            value: expenses_annual,
        },
        Slice {
            name: "Income tax",
            color: "#e06c75",
            value: income_tax_net,
        },
        Slice {
            name: "Medicare (levy + surcharge)",
            color: "#61afef",
            value: medicare,
        },
        Slice {
            name: "Study loan",
            color: "#c678dd",
            value: result.help_repayment_annual,
        },
        Slice {
            name: "Salary sacrifice + extra super",
            color: "#e5c07b",
            value: sacrificed,
        },
    ]
    .into_iter()
    .filter(|s| s.value > 0.005)
    .collect();

    let total: f64 = slices.iter().map(|s| s.value).sum();
    if total <= 0.0 {
        return view! { <section class="chart-block"></section> };
    }

    let cx = 130.0;
    let cy = 130.0;
    let r = 104.0;
    let mut angle = -TAU / 4.0;
    let paths = slices
        .iter()
        .map(|s| {
            let frac = s.value / total;
            let a0 = angle;
            let a1 = a0 + frac * TAU;
            angle = a1;
            if frac > 0.999 {
                view! { <circle cx={cx} cy={cy} r={r} fill={s.color} class="pie-slice" /> }
                    .into_view()
            } else {
                let (x0, y0) = (cx + r * a0.cos(), cy + r * a0.sin());
                let (x1, y1) = (cx + r * a1.cos(), cy + r * a1.sin());
                let large = if (a1 - a0) > TAU / 2.0 { 1 } else { 0 };
                let d =
                    format!("M{cx},{cy} L{x0:.2},{y0:.2} A{r},{r} 0 {large} 1 {x1:.2},{y1:.2} Z");
                view! { <path d={d} fill={s.color} class="pie-slice" /> }.into_view()
            }
        })
        .collect_view();

    let legend = slices
        .iter()
        .map(|s| {
            let pct = s.value / total * 100.0;
            view! {
                <div class="hover-row">
                    <span class="legend-swatch" style={format!("background:{}", s.color)}></span>
                    <span>{s.name}</span>
                    <strong>{format!("{} ({:.1}%)", fmt_money(s.value), pct)}</strong>
                </div>
            }
        })
        .collect_view();

    view! {
        <section class="chart-block">
            <Collapsible title="Where Your Gross Income Goes">
            <div class="pie-wrap">
                <svg
                    viewBox="0 0 260 260"
                    class="pie-chart"
                    role="img"
                    aria-label="annual gross income distribution pie chart"
                >
                    {paths}
                </svg>
                <div class="hover-readout pie-legend">
                    <div class="hover-row">
                        <span>"Gross income (annual)"</span>
                        <strong>{fmt_money(gross)}</strong>
                    </div>
                    {legend}
                    {has_outgoings.then(|| view! {
                        <p class="muted pie-note">
                            "Mortgage repayments are the first-year total from the Mortgages tab; debt recycling is the monthly redraw from the Debt Recycling tab; budget expenses come from the Budget tab."
                        </p>
                    })}
                </div>
            </div>
            </Collapsible>
        </section>
    }
}

#[derive(Clone, PartialEq)]
pub struct RatePoint {
    pub gross: f64,
    pub effective: f64,
    pub marginal: f64,
}

#[component]
pub fn RateCurveChart(
    points: Vec<RatePoint>,
    current_gross: f64,
    current_effective: f64,
    current_marginal: f64,
) -> impl IntoView {
    let width = 1040.0;
    let height = 420.0;
    let left_pad = 80.0;
    let right_pad = 24.0;
    let top_pad = 24.0;
    let bottom_pad = 64.0;
    let chart_w = width - left_pad - right_pad;
    let chart_h = height - top_pad - bottom_pad;

    let x_max = points
        .iter()
        .map(|p| p.gross)
        .fold(1.0_f64, f64::max)
        .max(current_gross);
    let y_data_max = points
        .iter()
        .map(|p| p.marginal.max(p.effective))
        .fold(0.0_f64, f64::max)
        .max(current_marginal);
    let y_max = ((y_data_max / 5.0).ceil() * 5.0).max(20.0);

    let x_of = move |g: f64| left_pad + (g / x_max).clamp(0.0, 1.0) * chart_w;
    let y_of = move |rate: f64| top_pad + chart_h - (rate / y_max).clamp(0.0, 1.0) * chart_h;

    let y_step = if y_max <= 25.0 { 5.0 } else { 10.0 };
    let y_ticks = (y_max / y_step).round() as usize;
    let y_grid = (0..=y_ticks)
        .map(|i| {
            let value = i as f64 * y_step;
            let y = y_of(value);
            view! {
                <g>
                    <line x1={left_pad} y1={y} x2={left_pad + chart_w} y2={y} class="axis-grid" />
                    <text x={left_pad - 10.0} y={y + 4.0} text-anchor="end" class="axis-label">
                        {format!("{value:.0}%")}
                    </text>
                </g>
            }
        })
        .collect_view();

    let x_ticks = 8_usize;
    let x_grid = (0..=x_ticks)
        .map(|i| {
            let t = i as f64 / x_ticks as f64;
            let x = left_pad + t * chart_w;
            view! {
                <g>
                    <line x1={x} y1={top_pad} x2={x} y2={top_pad + chart_h} class="axis-grid" />
                    <text x={x} y={top_pad + chart_h + 24.0} text-anchor="middle" class="axis-label">
                        {fmt_currency(x_max * t)}
                    </text>
                </g>
            }
        })
        .collect_view();

    let polyline_points = |f: &dyn Fn(&RatePoint) -> f64| {
        points
            .iter()
            .map(|p| format!("{:.1},{:.1}", x_of(p.gross), y_of(f(p))))
            .collect::<Vec<_>>()
            .join(" ")
    };
    let marginal_pts = polyline_points(&|p| p.marginal);
    let effective_pts = polyline_points(&|p| p.effective);

    let marker_x = x_of(current_gross);

    view! {
        <section class="chart-block">
            <Collapsible title="Tax Rates Across Income">
            <svg
                viewBox="0 0 1040 420"
                class="line-chart"
                role="img"
                aria-label="marginal and effective tax rate versus gross income"
            >
                {y_grid}
                {x_grid}
                <rect x={left_pad} y={top_pad} width={chart_w} height={chart_h} fill="none" class="axis" />
                <polyline fill="none" points={marginal_pts} stroke="#e06c75" class="series-line" />
                <polyline fill="none" points={effective_pts} stroke="#98c379" class="series-line" />
                <line
                    x1={marker_x}
                    y1={top_pad}
                    x2={marker_x}
                    y2={top_pad + chart_h}
                    stroke="#ffb000"
                    stroke-width="1.5"
                    stroke-dasharray="4 4"
                />
            </svg>
            <div class="hover-readout">
                <div class="hover-row">
                    <span class="legend-swatch" style="background:#ffb000"></span>
                    <span>"Your gross income"</span>
                    <strong>{fmt_currency(current_gross)}</strong>
                </div>
                <div class="hover-row">
                    <span class="legend-swatch" style="background:#e06c75"></span>
                    <span>"Marginal rate"</span>
                    <strong>{format!("{current_marginal:.0}%")}</strong>
                </div>
                <div class="hover-row">
                    <span class="legend-swatch" style="background:#98c379"></span>
                    <span>"Effective rate"</span>
                    <strong>{format!("{current_effective:.2}%")}</strong>
                </div>
            </div>
            </Collapsible>
        </section>
    }
}
