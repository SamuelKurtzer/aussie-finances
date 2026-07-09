use leptos::*;
use wasm_bindgen::JsCast;

use crate::formatting::fmt_currency;

#[derive(Clone)]
pub struct ChartLine {
    pub name: String,
    pub color: &'static str,
    pub values: Vec<f64>,
    pub opacity: f64,
    pub dashed: bool,
}

fn axis_step(value: f64) -> f64 {
    if value <= 1.0 {
        return 1.0;
    }
    let target_ticks = 10.0;
    let raw = (value / target_ticks).max(1.0);
    let exponent = raw.log10().floor();
    let magnitude = 10_f64.powf(exponent);
    let normalized = raw / magnitude;
    let nice = if normalized <= 1.0 {
        1.0
    } else if normalized <= 2.0 {
        2.0
    } else if normalized <= 5.0 {
        5.0
    } else {
        10.0
    };
    nice * magnitude
}

#[component]
pub fn MultiLineChart(
    title: String,
    period_months: Vec<f64>,
    lines: Vec<ChartLine>,
) -> impl IntoView {
    let width = 1040.0;
    let height = 460.0;
    let left_pad = 136.0;
    let right_pad = 24.0;
    let top_pad = 30.0;
    let bottom_pad = 74.0;

    let n_points = lines
        .iter()
        .map(|l| l.values.len())
        .max()
        .unwrap_or(0)
        .max(2);
    let n = n_points as f64;
    let data_max_x = period_months
        .last()
        .copied()
        .unwrap_or((n_points.saturating_sub(1)) as f64);
    let axis_max_x = data_max_x.max(1.0);

    let chart_w = width - left_pad - right_pad;
    let chart_h = height - top_pad - bottom_pad;

    let default_idx = period_months
        .iter()
        .enumerate()
        .min_by(|(_, a), (_, b)| {
            ((*a - 12.0).abs())
                .partial_cmp(&((*b - 12.0).abs()))
                .unwrap_or(std::cmp::Ordering::Equal)
        })
        .map(|(i, _)| i)
        .unwrap_or(0)
        .min(n_points.saturating_sub(1));
    let hover_idx = create_rw_signal(Some(default_idx));
    let line_enabled: Vec<RwSignal<bool>> =
        (0..lines.len()).map(|_| create_rw_signal(true)).collect();
    let lines_for_series = lines.clone();
    let enabled_for_series = line_enabled.clone();
    let lines_for_scale = lines.clone();
    let enabled_for_scale = line_enabled.clone();
    let lines_for_hover = lines.clone();
    let enabled_for_hover = line_enabled.clone();
    let enabled_for_toggles = line_enabled.clone();
    let months_for_hover_line = period_months.clone();
    let months_for_hover_readout = period_months.clone();
    let months_for_mouse = period_months.clone();

    let y_axis = create_memo(move |_| {
        let max_v = lines_for_scale
            .iter()
            .enumerate()
            .filter(|(idx, _)| enabled_for_scale[*idx].get())
            .flat_map(|(_, l)| l.values.iter().copied())
            .fold(0.0_f64, f64::max)
            .max(1.0);
        let y_headroom = max_v * 1.05;
        let y_step = axis_step(y_headroom);
        let y_ticks = (y_headroom / y_step).ceil().max(1.0) as usize;
        let axis_max = y_step * y_ticks as f64;
        (axis_max, y_step, y_ticks)
    });

    let y_grid = move || {
        let (_axis_max, y_step, y_ticks) = y_axis.get();
        (0..=y_ticks)
            .map(|i| {
                let value = (y_ticks - i) as f64 * y_step;
                let y = top_pad + (i as f64 / y_ticks as f64) * chart_h;
                view! {
                    <g>
                        <line x1={left_pad} y1={y} x2={left_pad + chart_w} y2={y} class="axis-grid" />
                        <text x={left_pad - 12.0} y={y + 4.0} text-anchor="end" class="axis-label">{fmt_currency(value)}</text>
                    </g>
                }
            })
            .collect_view()
    };

    let x_ticks = 12_usize;
    let x_grid = (0..=x_ticks)
        .map(|i| {
            let t = i as f64 / x_ticks as f64;
            let x = left_pad + t * chart_w;
            let month = axis_max_x * t;
            let label = format!("M{}", month.round() as i64);
            view! {
                <g>
                    <line x1={x} y1={top_pad} x2={x} y2={top_pad + chart_h} class="axis-grid" />
                    <text x={x} y={top_pad + chart_h + 24.0} text-anchor="middle" class="axis-label">{label}</text>
                </g>
            }
        })
        .collect_view();

    let series = move || {
        let (axis_max, _y_step, _y_ticks) = y_axis.get();
        lines_for_series
            .iter()
            .enumerate()
            .filter_map(|(line_idx, line)| {
                if !enabled_for_series[line_idx].get() {
                    return None;
                }
                let points = line
                    .values
                    .iter()
                    .enumerate()
                    .map(|(i, v)| {
                        let month = period_months
                            // series path uses the base vector directly
                            .get(i)
                            .copied()
                            .unwrap_or_else(|| i as f64 * (data_max_x / (n - 1.0).max(1.0)));
                        let x = left_pad + ((month / axis_max_x).clamp(0.0, 1.0)) * chart_w;
                        let y = top_pad + chart_h - ((*v / axis_max) * chart_h);
                        format!("{x:.1},{y:.1}")
                    })
                    .collect::<Vec<_>>()
                    .join(" ");

                Some(view! {
                    <polyline
                        fill="none"
                        points={points}
                        stroke={line.color}
                        class="series-line"
                        stroke-opacity={line.opacity}
                        stroke-dasharray={if line.dashed { "8 6" } else { "none" }}
                    />
                })
            })
            .collect_view()
    };

    let hover_line = move || {
        hover_idx
            .get()
            .map(|idx| {
                let month = months_for_hover_line
                    .get(idx)
                    .copied()
                    .unwrap_or_else(|| idx as f64 * (data_max_x / (n - 1.0).max(1.0)));
                let x = left_pad + ((month / axis_max_x).clamp(0.0, 1.0)) * chart_w;
                view! {
                    <line x1={x} y1={top_pad} x2={x} y2={top_pad + chart_h} class="hover-guide" />
                }
                .into_view()
            })
            .unwrap_or_else(|| view! { <g></g> }.into_view())
    };

    let hover_readout = move || {
        let idx = hover_idx.get().unwrap_or(default_idx);
        let label = months_for_hover_readout
            .get(idx)
            .copied()
            .map(|m| format!("Month {}", m.round() as i64))
            .unwrap_or_else(|| format!("Month {idx}"));
        view! {
            <div class="hover-readout">
                <strong>{label}</strong>
                {lines_for_hover
                    .iter()
                    .enumerate()
                    .map(|(line_idx, line)| {
                        let enabled = enabled_for_toggles[line_idx];
                        if !enabled_for_hover[line_idx].get() {
                            return view! {
                                <div class="hover-row muted">
                                    <input
                                        type="checkbox"
                                        prop:checked=move || enabled.get()
                                        on:change=move |ev| enabled.set(event_target_checked(&ev))
                                    />
                                    <span class="legend-swatch" style={format!("background:{}; opacity:{}", line.color, line.opacity)}></span>
                                    <span>{line.name.clone()}</span>
                                    <strong>"Hidden"</strong>
                                </div>
                            };
                        }
                        let value = line.values.get(idx).copied().unwrap_or(0.0);
                        view! {
                            <div class="hover-row">
                                <input
                                    type="checkbox"
                                    prop:checked=move || enabled.get()
                                    on:change=move |ev| enabled.set(event_target_checked(&ev))
                                />
                                <span class="legend-swatch" style={format!("background:{}; opacity:{}", line.color, line.opacity)}></span>
                                <span>{line.name.clone()}</span>
                                <strong>{fmt_currency(value)}</strong>
                            </div>
                        }
                    })
                    .collect_view()}
            </div>
        }
            .into_view()
    };

    let on_move = move |ev: leptos::ev::PointerEvent| {
        let rect = ev
            .target()
            .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
            .map(|el| el.get_bounding_client_rect())
            .or_else(|| {
                ev.current_target()
                    .and_then(|t| t.dyn_into::<web_sys::Element>().ok())
                    .map(|el| el.get_bounding_client_rect())
            });
        let Some(rect) = rect else {
            return;
        };
        if rect.width() <= 0.0 {
            return;
        }
        let normalized = ((ev.client_x() as f64 - rect.left()) / rect.width()).clamp(0.0, 1.0);
        let month = axis_max_x * normalized;
        let idx = months_for_mouse
            .iter()
            .enumerate()
            .min_by(|(_, a), (_, b)| {
                ((*a - month).abs())
                    .partial_cmp(&((*b - month).abs()))
                    .unwrap_or(std::cmp::Ordering::Equal)
            })
            .map(|(i, _)| i)
            .unwrap_or_else(|| ((n - 1.0) * normalized).round() as usize);
        hover_idx.set(Some(idx.min(n_points.saturating_sub(1))));
    };

    view! {
        <section class="chart-block">
            <h3>{title}</h3>

            <svg
                viewBox="0 0 1040 460"
                class="line-chart"
                role="img"
                aria-label="mortgage trend chart"
            >
                {y_grid}
                {x_grid}
                <rect x={left_pad} y={top_pad} width={chart_w} height={chart_h} fill="none" class="axis" />
                {series}
                <rect
                    x={left_pad}
                    y={top_pad}
                    width={chart_w}
                    height={chart_h}
                    fill="transparent"
                    pointer-events="all"
                    class="hover-capture"
                    on:pointerdown=on_move.clone()
                    on:pointermove=on_move
                />
                {hover_line}
            </svg>

            {hover_readout}
        </section>
    }
}
