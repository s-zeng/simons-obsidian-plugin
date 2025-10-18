use rust::{
    PluginError, PluginSettings, deserialize_settings, serialize_settings,
    validate_setting_internal,
};

#[test]
fn test_default_settings_serialization() {
    let settings = PluginSettings::default();
    let json = serialize_settings(&settings).expect("Should serialize default settings");
    insta::assert_snapshot!(json, @r#"{"mySetting":"default"}"#);
}

#[test]
fn test_deserialize_valid_settings() {
    let json = r#"{"mySetting":"custom_value"}"#;
    let settings = deserialize_settings(json).expect("Should deserialize valid JSON");
    insta::assert_snapshot!(settings.my_setting, @"custom_value");
}

#[test]
fn test_deserialize_invalid_json() {
    let json = r"invalid json";
    let result = deserialize_settings(json);
    assert!(result.is_err());

    if let Err(PluginError::SerializationError { context, source }) = result {
        insta::assert_snapshot!(context, @"deserialize_settings");
        // Source error message contains specific parse error - just verify it exists
        assert!(!source.is_empty());
    } else {
        panic!("Expected SerializationError");
    }
}

#[test]
fn test_validate_setting_my_setting_valid() {
    let result = validate_setting_internal("mySetting", "some_value");
    assert!(result.is_ok());
}

#[test]
fn test_validate_setting_my_setting_empty() {
    let result = validate_setting_internal("mySetting", "");
    assert!(result.is_err());

    if let Err(PluginError::ValidationError { field, value, reason }) = result {
        insta::assert_snapshot!(field, @"mySetting");
        insta::assert_snapshot!(value, @"");
        insta::assert_snapshot!(reason, @"Setting value cannot be empty");
    } else {
        panic!("Expected ValidationError");
    }
}

#[test]
fn test_validate_setting_unknown_key() {
    let result = validate_setting_internal("unknownKey", "value");
    assert!(result.is_err());

    if let Err(PluginError::UnknownSetting { key }) = result {
        insta::assert_snapshot!(key, @"unknownKey");
    } else {
        panic!("Expected UnknownSetting error");
    }
}

#[test]
fn test_plugin_error_display_validation() {
    let error = PluginError::ValidationError {
        field: "testField".to_string(),
        value: "testValue".to_string(),
        reason: "test reason".to_string(),
    };
    let display = error.to_string();
    insta::assert_snapshot!(display, @"Validation failed for field 'testField' with value 'testValue': test reason");
}

#[test]
fn test_plugin_error_display_serialization() {
    let error = PluginError::SerializationError {
        context: "test_context".to_string(),
        source: "test source error".to_string(),
    };
    let display = error.to_string();
    insta::assert_snapshot!(display, @"Serialization error in test_context: test source error");
}

#[test]
fn test_plugin_error_display_unknown_setting() {
    let error = PluginError::UnknownSetting { key: "unknownKey".to_string() };
    let display = error.to_string();
    insta::assert_snapshot!(display, @"Unknown setting key: 'unknownKey'");
}
