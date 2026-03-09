# aus-fin

Static Leptos website for Australian personal finance tools.

## Scope (MVP)

- Income calculator (gross-to-net, resident-only, FY 2025-26 rules).
- Detailed annual + per-pay breakdown.
- Mortgage planner:
- Multiple mortgages with multiple splits per mortgage.
- Inputs for loan amount, rate, equity, loan purpose, rate type, and IO/P&I behavior.
- Projection outputs including summary metrics, amortization table, and line charts.
- Offset top-up per repayment period and repayment-to-net-income percentage.
- Local browser persistence for both income and mortgage tabs.
- Placeholder guide pages for tax and debt recycling.

## Run locally

Prerequisites:

- Rust toolchain
- `trunk` (`cargo install trunk`)

Commands:

```bash
cargo test
trunk serve --open
```

## Build for static hosting

```bash
trunk build --release
```

The static output is in `dist/`.

## DigitalOcean Static Site deployment

1. Create a Static Site in DigitalOcean App Platform.
2. Connect your Git repository.
3. Configure build command:
   - `trunk build --release`
4. Configure output directory:
   - `dist`
5. Configure SPA fallback rewrite:
   - route `/*` to `/index.html`
6. Set deployment to trigger on main branch pushes.

## Disclaimer

This calculator provides an estimate only and is not financial advice.
