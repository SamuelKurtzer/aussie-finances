# Technical Debt Backlog

Last reviewed: 2026-07-09.

## Shipped since last review

- Tax year selector (FY 2024-25 and 2025-26, table-driven `TaxRules::for_year`).
- Non-resident and working holiday maker tax models.
- LITO, SAPTO, Medicare levy low-income phase-in, MLS tiers, Division 293.
- CSV export (spreadsheet tab) and JSON backup export/import (header).
- Net-to-gross reverse solver on the income tab.
- Chart hover/touch readouts on line charts.
- Tauri desktop and Android shells; collapsible sections; mobile layout.

## High priority

1. Mortgage rate schedule support
- Projections assume constant rates per split; fixed-rate expiry into a
  variable rate is the most common real scenario.
- Path: future-dated rate change schedule per split.

2. Mortgage redraw and transaction cashflow model
- Only offset growth is modelled; no irregular extra repayments, redraw
  usage, or fees.
- Path: period-based cashflow events per split.

## Medium priority

1. PDF export of a results summary (CSV/JSON exist).
2. Android release signing (keystore + `key.properties`) for installable
   release builds; debug builds only today.
3. PWA manifest + service worker so the web build installs and works
   offline like the Tauri shells.
4. Collapsible-section open/closed state (`aus_fin_collapse_*`) is not
   included in JSON backups; only core data keys are.

## Known approximation debt

1. HELP/HECS schedule maintenance
- Static tables in `tax_rules.rs`; needs an annual update workflow.

2. FY 2025-26 Medicare levy thresholds
- Values are projections until legislated; revisit after the budget.

3. Cliff behavior in the reverse solver
- MLS and FY 2024-25 HELP apply to whole income, so net pay is not
  monotonic in gross. The solver scans for the first crossing at ~$400
  granularity; dips narrower than the scan step could be skipped.

4. Repayment-to-income ratio cadence handling
- Ratio uses annual net income normalized to mortgage cadence; no
  explicit cadence mapping controls.
