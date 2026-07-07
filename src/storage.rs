pub const INCOME_STORAGE_KEY: &str = "aus_fin_income_calculator_v1";
pub const MORTGAGE_STORAGE_KEY: &str = "aus_fin_mortgage_calculator_v1";
pub const DEBT_RECYCLE_STORAGE_KEY: &str = "aus_fin_debt_recycle_v1";
pub const BUDGET_STORAGE_KEY: &str = "aus_fin_budget_v1";
pub const ACTIVE_TAB_STORAGE_KEY: &str = "aus_fin_active_tab_v1";

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
