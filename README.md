# aus-fin

Static Leptos website for Australian personal finance tools. Also ships as a
desktop and Android app via Tauri (`src-tauri/`).

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

## Desktop app (Tauri)

Prerequisites (Arch): `webkit2gtk-4.1`, `gtk3`, and the Tauri CLI
(`cargo binstall tauri-cli` or `cargo install tauri-cli --locked`).

```bash
cargo tauri dev      # dev window with hot reload (runs `trunk serve` for you)
cargo tauri build    # release bundles in target/release/bundle/ (deb, rpm, AppImage)
```

The dev server is pinned to port 1420 in `Trunk.toml` to match `devUrl` in
`src-tauri/tauri.conf.json`.

## Android app (Tauri)

One-time setup:

```bash
rustup target add aarch64-linux-android armv7-linux-androideabi \
    i686-linux-android x86_64-linux-android

# SDK layout must be cmdline-tools/latest/bin for sdkmanager to work
export ANDROID_HOME="$HOME/Android/Sdk"
export JAVA_HOME=/usr/lib/jvm/java-21-openjdk
"$ANDROID_HOME/cmdline-tools/latest/bin/sdkmanager" \
    "platform-tools" "platforms;android-34" "build-tools;34.0.0" "ndk;27.2.12479018"
export NDK_HOME="$ANDROID_HOME/ndk/27.2.12479018"

cargo tauri android init   # generates src-tauri/gen/android
```

Run and build:

```bash
cargo tauri android dev    # deploys to a connected device or running emulator
cargo tauri android build  # release APK/AAB in src-tauri/gen/android/app/build/outputs/
```

iOS requires macOS with Xcode and cannot be built from Linux.

## DigitalOcean Static Site deployment

1. Create a Static Site in DigitalOcean App Platform.
2. Connect your Git repository.
3. Configure build command:
   - `trunk build --release`
4. Configure output directory:
   - `dist`
5. Configure SPA fallback rewrite:
   - route `/*` to `/index.html`
   - If a rewrite is not configured, the build also emits `dist/404.html`
     (a copy of `index.html`), which static hosts serve for unknown paths so
     deep links like `/income` still load the app.
6. Set deployment to trigger on main branch pushes.

## Disclaimer

This calculator provides an estimate only and is not financial advice.
