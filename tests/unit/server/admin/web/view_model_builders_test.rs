//! Unit tests for view model builder helpers
//!
//! Tests for helpers that reduce duplication in ViewModelBuilder.

use mcp_context_browser::application::admin::helpers::activity::ActivityLevel;
use mcp_context_browser::server::admin::web::view_model_builders::{
    ActivityLevelFormatter, ConfigCategoryBuilder, ConfigSettingBuilder,
};

#[test]
fn test_config_setting_builder_number() {
    let setting = ConfigSettingBuilder::number("chunk_size", "Chunk Size", 512, "Size of chunks");
    assert_eq!(setting.key, "chunk_size");
    assert_eq!(setting.label, "Chunk Size");
    assert_eq!(setting.value_display, "512");
    assert_eq!(setting.setting_type, "number");
}

#[test]
fn test_config_setting_builder_boolean() {
    let setting =
        ConfigSettingBuilder::boolean("enable_auth", "Enable Auth", true, "Enable authentication");
    assert_eq!(setting.key, "enable_auth");
    assert_eq!(setting.value_display, "Enabled");
    assert_eq!(setting.setting_type, "boolean");

    let disabled =
        ConfigSettingBuilder::boolean("enable_auth", "Enable Auth", false, "Enable authentication");
    assert_eq!(disabled.value_display, "Disabled");
}

#[test]
fn test_config_setting_builder_bytes() {
    let setting =
        ConfigSettingBuilder::bytes("max_file_size", "Max File Size", 1_048_576, "Max size");
    assert_eq!(setting.value_display, "1.0 MB");
}

#[test]
fn test_config_category_builder() {
    let settings = vec![ConfigSettingBuilder::number(
        "key1",
        "Label 1",
        100,
        "Description 1",
    )];
    let category = ConfigCategoryBuilder::new("Test Category", "Test description", settings);
    assert_eq!(category.name, "Test Category");
    assert_eq!(category.description, "Test description");
    assert_eq!(category.settings.len(), 1);
}

#[test]
fn test_activity_level_formatter() {
    assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Success).contains("success"));
    assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Error).contains("error"));
    assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Warning).contains("warning"));
    assert!(ActivityLevelFormatter::to_css_class(ActivityLevel::Info).contains("info"));
}
