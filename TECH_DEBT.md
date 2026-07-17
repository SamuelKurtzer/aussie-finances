# Technical Debt Backlog

Last reviewed: 2026-07-17.

## Shipped since last review

- Fixed-rate expiry per mortgage split: fixed period in months plus revert
  rate, re-amortizing over the remaining term at rollover; the split becomes
  redraw-eligible for debt recycling afterwards.
- Debt recycle redraw cadence (monthly/quarterly/yearly) and income sweep so
  the offset pins to the emergency buffer.
- Dividends + franking credits in the income calculator (30% gross-up,
  refundable offset; negative Total Withheld = refund).
- Android release signing (upload keystore via env vars / gitignored
  key.properties); tags build a signed release APK.
- GitHub Releases on v* tags with desktop bundles and the APK attached.
- GitHub Pages deploy of the web build on main pushes; dist/ untracked.
- PWA manifest + network-first service worker (web installs, works offline).
- CI: pinned taiki-e/install-action (binstall@main breakage), fmt + clippy
  gates on the test job.
- Collapsible open/closed state included in JSON backups (prefix scan).
- Mobile layout: single-row tab bar, viewport-width pages, table scroll
  wrappers.

## High priority

1. Mortgage redraw and transaction cashflow model
- Only offset growth is modelled; no irregular extra repayments, redraw
  usage, or fees.
- Path: period-based cashflow events per split.

## Medium priority

1. PDF export of a results summary (CSV/JSON exist).
2. Generalize `FixedRateExpiry` to a `Vec<RateChange>` schedule if
   multi-step rate paths are ever needed; the single fixed-to-variable
   rollover covers the common case today.
3. Service worker cache pruning: old hashed assets linger in the runtime
   cache until the `CACHE` version constant in `static/sw.js` is bumped.
4. Franking gross-up assumes the 30% company rate; base-rate-entity (25%)
   franking is not modelled.

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

## Infrastructure notes

- `src-tauri/gen/android/app/build.gradle.kts` carries a hand-written
  release signing config; re-running `cargo tauri android init` would
  clobber it — re-apply from git history if the project is ever
  regenerated.
- The upload keystore lives outside the repo (`~/keys/aus-fin-upload.jks`
  locally, base64 secret in CI). Losing it breaks APK update continuity;
  keep a backup.
