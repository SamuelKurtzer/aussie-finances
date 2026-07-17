pub const INCOME_STORAGE_KEY: &str = "aus_fin_income_calculator_v1";
pub const MORTGAGE_STORAGE_KEY: &str = "aus_fin_mortgage_calculator_v1";
pub const DEBT_RECYCLE_STORAGE_KEY: &str = "aus_fin_debt_recycle_v1";
pub const BUDGET_STORAGE_KEY: &str = "aus_fin_budget_v1";
pub const ACTIVE_TAB_STORAGE_KEY: &str = "aus_fin_active_tab_v1";
/// Collapsible sections persist open/closed state under dynamic keys with
/// this prefix (one per section title); backups pick them up by prefix scan.
pub const COLLAPSE_KEY_PREFIX: &str = "aus_fin_collapse_";

/// Every key included in backup export/import.
pub const ALL_BACKUP_KEYS: &[&str] = &[
    INCOME_STORAGE_KEY,
    MORTGAGE_STORAGE_KEY,
    DEBT_RECYCLE_STORAGE_KEY,
    BUDGET_STORAGE_KEY,
    ACTIVE_TAB_STORAGE_KEY,
];

#[cfg(target_arch = "wasm32")]
pub fn load_from_storage<T: serde::de::DeserializeOwned>(key: &str) -> Option<T> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    let raw = storage.get_item(key).ok()??;
    serde_json::from_str::<T>(&raw).ok()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_from_storage<T: serde::de::DeserializeOwned>(_key: &str) -> Option<T> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn load_raw_from_storage(key: &str) -> Option<String> {
    let window = web_sys::window()?;
    let storage = window.local_storage().ok()??;
    storage.get_item(key).ok()?
}

#[cfg(not(target_arch = "wasm32"))]
pub fn load_raw_from_storage(_key: &str) -> Option<String> {
    None
}

#[cfg(target_arch = "wasm32")]
pub fn save_raw_to_storage(key: &str, raw: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.set_item(key, raw);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
#[allow(dead_code)]
pub fn save_raw_to_storage(_key: &str, _raw: &str) {}

#[cfg(target_arch = "wasm32")]
pub fn save_to_storage<T: serde::Serialize>(key: &str, value: &T) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            if let Ok(serialized) = serde_json::to_string(value) {
                let _ = storage.set_item(key, &serialized);
            }
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn save_to_storage<T: serde::Serialize>(_key: &str, _value: &T) {}

/// RwSignal initialized from localStorage and written back on every change.
pub fn persisted_signal<T>(key: &'static str) -> leptos::RwSignal<T>
where
    T: serde::Serialize + serde::de::DeserializeOwned + Clone + Default + 'static,
{
    use leptos::{create_effect, create_rw_signal, SignalWith};
    let signal = create_rw_signal(load_from_storage::<T>(key).unwrap_or_default());
    create_effect(move |_| signal.with(|value| save_to_storage(key, value)));
    signal
}

#[cfg(target_arch = "wasm32")]
pub fn list_keys_with_prefix(prefix: &str) -> Vec<String> {
    let Some(window) = web_sys::window() else {
        return Vec::new();
    };
    let Ok(Some(storage)) = window.local_storage() else {
        return Vec::new();
    };
    let len = storage.length().unwrap_or(0);
    (0..len)
        .filter_map(|i| storage.key(i).ok().flatten())
        .filter(|key| key.starts_with(prefix))
        .collect()
}

#[cfg(not(target_arch = "wasm32"))]
pub fn list_keys_with_prefix(_prefix: &str) -> Vec<String> {
    Vec::new()
}

#[cfg(target_arch = "wasm32")]
pub fn remove_from_storage(key: &str) {
    if let Some(window) = web_sys::window() {
        if let Ok(Some(storage)) = window.local_storage() {
            let _ = storage.remove_item(key);
        }
    }
}

#[cfg(not(target_arch = "wasm32"))]
pub fn remove_from_storage(_key: &str) {}
