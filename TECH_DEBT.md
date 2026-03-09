# Technical Debt Backlog

## High priority

1. Tax year selector
- Status: deferred by MVP scope.
- Rationale: v1 locked to FY 2025-26 to ship quickly.
- Impact: users cannot model prior/future tax years.
- Path: move `TaxRules::fy_2025_26_resident()` to table-driven `TaxRulesSet` keyed by year; add year selector to UI and persist selection in route query params.

2. Non-resident and working holiday maker tax models
- Status: deferred by residency scope.
- Rationale: resident-only logic is currently hard-coded.
- Impact: calculator unsuitable for non-resident cases.
- Path: add residency enum and separate tax/MLS/HELP strategies.

3. Offsets and rebates
- Status: deferred due rules complexity.
- Rationale: current estimator excludes offsets.
- Impact: may overstate withholding for eligible users.
- Path: add offset engine post-tax with eligibility rules.

## Medium priority

1. Export support
- Status: deferred.
- Rationale: MVP focuses on on-screen transparency.
- Impact: no downloadable summary for record keeping.
- Path: add CSV export first, then PDF.

2. Reverse mode (net-to-gross)
- Status: deferred.
- Rationale: gross-to-net chosen for first release.
- Impact: users cannot solve for required gross salary.
- Path: implement iterative solver with convergence guards.

3. Mortgage rate schedule support
- Status: deferred.
- Rationale: mortgage projections currently assume constant rates per split.
- Impact: forecasts may diverge from real outcomes when rates change.
- Path: add future-dated rate change schedule per split.

4. Mortgage redraw and transaction cashflow model
- Status: deferred.
- Rationale: current mortgage model supports offset growth only.
- Impact: does not reflect redraw usage or irregular extra repayments.
- Path: add period-based cashflow events (extra repayments, redraw, fees).

5. Mortgage chart interactivity
- Status: deferred.
- Rationale: charts are static SVG lines in v1.
- Impact: no hover detail, zoom, or custom period ranges.
- Path: add interactive chart layer with tooltips and selectable horizons.

## Known approximation debt

1. Medicare low-income reduction is simplified
- Current behavior: flat threshold gate then full 2% rate.
- Path: replace with full phase-in formula and family thresholds.

2. HELP/HECS schedule maintenance
- Current behavior: static table in code for MVP.
- Path: annual rules update workflow and data-file driven thresholds.

3. Repayment-to-income ratio cadence handling
- Current behavior: ratio uses annual net income normalized to mortgage cadence.
- Path: add explicit cadence mapping controls and show side-by-side cadence-adjusted assumptions.
