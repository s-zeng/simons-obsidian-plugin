use serde::{Deserialize, Serialize};
use wasm_bindgen::prelude::*;
use crate::error::PluginError;

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

/// Internal validation function with proper error types
pub fn validate_setting_internal(key: &str, value: &str) -> Result<(), PluginError> {
    match key {
        "mySetting" => {
            if value.is_empty() {
                return Err(PluginError::ValidationError {
                    field: key.to_string(),
                    value: value.to_string(),
                    reason: "Setting value cannot be empty".to_string(),
                });
            }
            Ok(())
        }
        _ => Err(PluginError::UnknownSetting {
            key: key.to_string(),
        }),
    }
}

/// Internal serialization function with proper error handling
pub fn serialize_settings(settings: &PluginSettings) -> Result<String, PluginError> {
    serde_json::to_string(settings).map_err(|e| PluginError::SerializationError {
        context: "serialize_settings".to_string(),
        source: e.to_string(),
    })
}

/// Internal deserialization function with proper error handling
pub fn deserialize_settings(json: &str) -> Result<PluginSettings, PluginError> {
    serde_json::from_str(json).map_err(|e| PluginError::SerializationError {
        context: "deserialize_settings".to_string(),
        source: e.to_string(),
    })
}

// WASM boundary functions - these convert between Result<T, PluginError> and JsValue

#[wasm_bindgen]
pub fn get_default_settings() -> String {
    let settings = PluginSettings::default();
    serialize_settings(&settings).unwrap_or_else(|_| "{}".to_string())
}

#[wasm_bindgen]
pub fn validate_setting(key: &str, value: &str) -> Result<(), JsValue> {
    validate_setting_internal(key, value).map_err(|e| e.into())
}

#[wasm_bindgen]
pub fn merge_settings(defaults: &str, loaded: &str) -> String {
    let default_settings: PluginSettings = deserialize_settings(defaults)
        .unwrap_or_else(|_| PluginSettings::default());

    let mut merged = default_settings.clone();

    if let Ok(loaded_settings) = serde_json::from_str::<serde_json::Value>(loaded) {
        if let Some(my_setting) = loaded_settings.get("mySetting") {
            if let Some(value) = my_setting.as_str() {
                merged.my_setting = value.to_string();
            }
        }
    }

    serialize_settings(&merged).unwrap_or_else(|_| defaults.to_string())
}
