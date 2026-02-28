use super::*;
use mcb_domain::error::Error;
use milvus::data::FieldColumn;

use milvus::schema::{CollectionSchema, CollectionSchemaBuilder, FieldSchema};
use milvus::value::Value;

use crate::constants::{
    MILVUS_FIELD_VARCHAR_MAX_LENGTH, MILVUS_METADATA_VARCHAR_MAX_LENGTH, VECTOR_FIELD_CONTENT,
    VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_ID, VECTOR_FIELD_START_LINE, VECTOR_FIELD_VECTOR,
};

pub(super) fn extract_field<T, F>(
    fields: &[FieldColumn],
    name: &str,
    index: usize,
    field_type: &str,
    extractor: F,
) -> Result<T>
where
    F: Fn(Value<'_>) -> Option<T>,
{
    fields
        .iter()
        .find(|column| column.name == name)
        .and_then(|column| column.get(index))
        .and_then(extractor)
        .ok_or_else(|| {
            Error::vector_db(format!(
                "Milvus response missing {field_type} field '{name}' at index {index}"
            ))
        })
}

pub(super) fn extract_string_field(
    fields: &[FieldColumn],
    name: &str,
    index: usize,
) -> Result<String> {
    #[allow(clippy::wildcard_enum_match_arm)]
    extract_field(fields, name, index, "string", |value| match value {
        Value::String(text) => Some(text.to_string()),
        _ => None,
    })
}

pub(super) fn extract_long_field(fields: &[FieldColumn], name: &str, index: usize) -> Result<i64> {
    #[allow(clippy::wildcard_enum_match_arm)]
    extract_field(fields, name, index, "long", |value| match value {
        Value::Long(number) => Some(number),
        _ => None,
    })
}


pub(super) fn build_collection_schema(
    name: &CollectionId,
    dimensions: usize,
) -> Result<CollectionSchema> {
    let name_str = to_milvus_name(name);
    CollectionSchemaBuilder::new(&name_str, &format!("Collection for {name}"))
        .add_field(FieldSchema::new_primary_int64(
            VECTOR_FIELD_ID,
            "primary key field",
            true,
        ))
        .add_field(FieldSchema::new_float_vector(
            VECTOR_FIELD_VECTOR,
            "feature field",
            dimensions as i64,
        ))
        .add_field(FieldSchema::new_varchar(
            VECTOR_FIELD_FILE_PATH,
            "file path",
            MILVUS_FIELD_VARCHAR_MAX_LENGTH,
        ))
        .add_field(FieldSchema::new_int64(
            VECTOR_FIELD_START_LINE,
            "start line",
        ))
        .add_field(FieldSchema::new_varchar(
            VECTOR_FIELD_CONTENT,
            "content",
            MILVUS_METADATA_VARCHAR_MAX_LENGTH,
        ))
        .build()
        .map_err(|e| Error::vector_db(format!("Failed to create schema: {e}")))
}
