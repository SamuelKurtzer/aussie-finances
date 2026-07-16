use crate::storage::{load_raw_from_storage, save_raw_to_storage, ALL_BACKUP_KEYS};

/// Serialize every known storage key into a portable JSON document.
pub fn export_backup_json() -> String {
    let mut keys = serde_json::Map::new();
    for key in ALL_BACKUP_KEYS {
        if let Some(raw) = load_raw_from_storage(key) {
            keys.insert((*key).to_string(), serde_json::Value::String(raw));
        }
    }
    let backup = serde_json::json!({
        "app": "aus-fin",
        "version": 1,
        "keys": keys,
    });
    serde_json::to_string_pretty(&backup).unwrap_or_else(|_| "{}".to_string())
}

/// Restore known keys from a backup document. Returns how many keys were
/// applied. Unknown keys are ignored; malformed payloads are rejected
/// before anything is written so a bad file can't half-apply.
#[cfg_attr(not(target_arch = "wasm32"), allow(dead_code))]
pub fn apply_backup(text: &str) -> Result<usize, String> {
    let value: serde_json::Value =
        serde_json::from_str(text).map_err(|_| "file is not valid JSON".to_string())?;
    let keys = value
        .get("keys")
        .and_then(|k| k.as_object())
        .ok_or_else(|| "not an aus-fin backup (missing \"keys\")".to_string())?;

    let mut to_apply: Vec<(&str, &str)> = Vec::new();
    for key in ALL_BACKUP_KEYS {
        if let Some(raw) = keys.get(*key).and_then(|v| v.as_str()) {
            serde_json::from_str::<serde_json::Value>(raw)
                .map_err(|_| format!("data for {key} is corrupted"))?;
            to_apply.push((key, raw));
        }
    }
    if to_apply.is_empty() {
        return Err("no aus-fin data found in this file".to_string());
    }
    let applied = to_apply.len();
    for (key, raw) in to_apply {
        save_raw_to_storage(key, raw);
    }
    Ok(applied)
}

#[cfg(target_arch = "wasm32")]
pub fn trigger_download(content: &str, filename: &str, mime: &str) {
    use wasm_bindgen::JsCast;
    use wasm_bindgen::JsValue;

    let array = js_sys::Array::new();
    array.push(&JsValue::from_str(content));
    let opts = web_sys::BlobPropertyBag::new();
    opts.set_type(mime);
    let Ok(blob) = web_sys::Blob::new_with_str_sequence_and_options(&array, &opts) else {
        return;
    };
    let Ok(url) = web_sys::Url::create_object_url_with_blob(&blob) else {
        return;
    };

    let document = web_sys::window().unwrap().document().unwrap();
    let a = document
        .create_element("a")
        .unwrap()
        .dyn_into::<web_sys::HtmlAnchorElement>()
        .unwrap();
    a.set_href(&url);
    a.set_download(filename);
    a.style().set_property("display", "none").unwrap();
    document.body().unwrap().append_child(&a).unwrap();
    a.click();
    document.body().unwrap().remove_child(&a).unwrap();
    let _ = web_sys::Url::revoke_object_url(&url);
}

#[cfg(not(target_arch = "wasm32"))]
pub fn trigger_download(_content: &str, _filename: &str, _mime: &str) {}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn rejects_non_json() {
        assert!(apply_backup("not json").is_err());
    }

    #[test]
    fn rejects_missing_keys_object() {
        assert!(apply_backup(r#"{"app":"aus-fin"}"#).is_err());
    }

    #[test]
    fn rejects_file_with_no_recognized_keys() {
        assert!(apply_backup(r#"{"keys":{"unrelated":"{}"}}"#).is_err());
    }

    #[test]
    fn rejects_corrupted_payload_before_applying() {
        let doc = r#"{"keys":{"aus_fin_budget_v1":"{{{not json"}}"#;
        assert!(apply_backup(doc).is_err());
    }

    #[test]
    fn export_round_trips_structure() {
        // Off-wasm storage is a no-op, so export yields an empty keys map,
        // which import correctly rejects as "no data".
        let doc = export_backup_json();
        assert!(apply_backup(&doc).is_err());
    }
}
