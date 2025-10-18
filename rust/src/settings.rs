use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;

#[derive(Serialize, Deserialize, Clone, Debug)]
pub struct PluginSettings {
    #[serde(rename = "mySetting")]
    pub my_setting: String,
}

impl Default for PluginSettings {
    fn default() -> Self {
        PluginSettings {
            my_setting: "default".to_string(),
        }
    }
}

#[wasm_bindgen]
pub fn get_default_settings() -> String {
    let settings = PluginSettings::default();
    serde_json::to_string(&settings).unwrap_or_else(|_| "{}".to_string())
}

#[wasm_bindgen]
pub fn validate_setting(key: &str, value: &str) -> Result<(), JsValue> {
    match key {
        "mySetting" => {
            if value.is_empty() {
                return Err(JsValue::from_str("Setting cannot be empty"));
            }
            Ok(())
        }
        _ => Err(JsValue::from_str(&format!("Unknown setting key: {}", key))),
    }
}

#[wasm_bindgen]
pub fn merge_settings(defaults: &str, loaded: &str) -> String {
    let default_settings: PluginSettings = serde_json::from_str(defaults)
        .unwrap_or_else(|_| PluginSettings::default());

    let mut merged = default_settings.clone();

    if let Ok(loaded_settings) = serde_json::from_str::<serde_json::Value>(loaded) {
        if let Some(my_setting) = loaded_settings.get("mySetting") {
            if let Some(value) = my_setting.as_str() {
                merged.my_setting = value.to_string();
            }
        }
    }

    serde_json::to_string(&merged).unwrap_or_else(|_| defaults.to_string())
}
