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
    let snapshot = match deserialize_settings(json) {
        Ok(_) => "ok".to_string(),
        Err(PluginError::SerializationError { context, source }) => {
            format!("context={context} source_nonempty={}", !source.is_empty())
        },
        Err(err) => format!("unexpected={err}"),
    };
    insta::assert_snapshot!(snapshot, @"context=deserialize_settings source_nonempty=true");
}

#[test]
fn test_validate_setting_my_setting_valid() {
    let snapshot = match validate_setting_internal("mySetting", "some_value") {
        Ok(()) => "ok".to_string(),
        Err(err) => format!("error={err}"),
    };
    insta::assert_snapshot!(snapshot, @"ok");
}

#[test]
fn test_validate_setting_my_setting_empty() {
    let snapshot = match validate_setting_internal("mySetting", "") {
        Ok(()) => "ok".to_string(),
        Err(PluginError::ValidationError { field, value, reason }) => {
            format!("field={field} value={value} reason={reason}")
        },
        Err(err) => format!("unexpected={err}"),
    };
    insta::assert_snapshot!(
        snapshot,
        @"field=mySetting value= reason=Setting value cannot be empty"
    );
}

#[test]
fn test_validate_setting_unknown_key() {
    let snapshot = match validate_setting_internal("unknownKey", "value") {
        Ok(()) => "ok".to_string(),
        Err(PluginError::UnknownSetting { key }) => format!("unknown_key={key}"),
        Err(err) => format!("unexpected={err}"),
    };
    insta::assert_snapshot!(snapshot, @"unknown_key=unknownKey");
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
