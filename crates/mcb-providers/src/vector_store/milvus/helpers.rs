use super::*;
use mcb_domain::error::Error;
use mcb_domain::value_objects::Embedding;
use milvus::data::FieldColumn;
use milvus::proto::schema::DataType;
use milvus::value::ValueVec;
use std::collections::HashMap;

use crate::constants::{
    MILVUS_FIELD_VARCHAR_MAX_LENGTH, MILVUS_METADATA_VARCHAR_MAX_LENGTH, VECTOR_FIELD_CONTENT,
    VECTOR_FIELD_FILE_PATH, VECTOR_FIELD_LINE_NUMBER, VECTOR_FIELD_START_LINE, VECTOR_FIELD_VECTOR,
};

#[derive(Debug)]
pub(super) struct InsertPayload {
    pub(super) expected_dims: usize,
    pub(super) vectors_flat: Vec<f32>,
    pub(super) file_paths: Vec<String>,
    pub(super) start_lines: Vec<i64>,
    pub(super) contents: Vec<String>,
}

pub(super) fn validate_insert_input(vectors: &[Embedding], metadata_len: usize) -> Result<usize> {
    if vectors.is_empty() {
        return Err(Error::vector_db(
            "No vectors provided for insertion".to_owned(),
        ));
    }

    if vectors.len() != metadata_len {
        return Err(Error::vector_db(format!(
            "Vectors ({}) and metadata ({}) arrays must have the same length",
            vectors.len(),
            metadata_len
        )));
    }

    let expected_dims = vectors[0].dimensions;
    for (i, vector) in vectors.iter().enumerate() {
        if vector.dimensions != expected_dims {
            return Err(Error::vector_db(format!(
                "Vector at index {} has dimensions {} but expected {}",
                i, vector.dimensions, expected_dims
            )));
        }
    }

    Ok(expected_dims)
}

pub(super) fn prepare_insert_data(
    vectors: &[Embedding],
    metadata: &[HashMap<String, serde_json::Value>],
    expected_dims: usize,
) -> Result<InsertPayload> {
    let capacity = vectors.len();
    let mut payload = InsertPayload {
        expected_dims,
        vectors_flat: Vec::with_capacity(capacity * expected_dims),
        file_paths: Vec::with_capacity(capacity),
        start_lines: Vec::with_capacity(capacity),
        contents: Vec::with_capacity(capacity),
    };

    for (i, (embedding, meta)) in vectors.iter().zip(metadata.iter()).enumerate() {
        payload.vectors_flat.extend_from_slice(&embedding.vector);
        payload.file_paths.push(
            meta.get(VECTOR_FIELD_FILE_PATH)
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    Error::vector_db(format!(
                        "Metadata missing '{VECTOR_FIELD_FILE_PATH}' at index {i}"
                    ))
                })?
                .to_owned(),
        );
        payload.start_lines.push(
            meta.get(VECTOR_FIELD_START_LINE)
                .and_then(serde_json::Value::as_i64)
                .or_else(|| {
                    meta.get(VECTOR_FIELD_LINE_NUMBER)
                        .and_then(serde_json::Value::as_i64)
                })
                .ok_or_else(|| {
                    Error::vector_db(format!(
                        "Metadata missing '{VECTOR_FIELD_START_LINE}' at index {i}"
                    ))
                })?,
        );
        payload.contents.push(
            meta.get(VECTOR_FIELD_CONTENT)
                .and_then(|value| value.as_str())
                .ok_or_else(|| {
                    Error::vector_db(format!(
                        "Metadata missing '{VECTOR_FIELD_CONTENT}' at index {i}"
                    ))
                })?
                .to_owned(),
        );
    }

    Ok(payload)
}

pub(super) fn build_field_column(
    name: &str,
    dtype: DataType,
    value: ValueVec,
    max_length: i32,
) -> FieldColumn {
    FieldColumn {
        name: name.to_owned(),
        dtype,
        value,
        dim: 1,
        max_length,
        is_dynamic: false,
    }
}

pub(super) fn build_insert_columns(payload: InsertPayload) -> Vec<FieldColumn> {
    let vector_column = FieldColumn {
        name: VECTOR_FIELD_VECTOR.to_owned(),
        dtype: DataType::FloatVector,
        value: ValueVec::Float(payload.vectors_flat),
        dim: payload.expected_dims as i64,
        max_length: 0,
        is_dynamic: false,
    };

    vec![
        vector_column,
        build_field_column(
            VECTOR_FIELD_FILE_PATH,
            DataType::VarChar,
            ValueVec::String(payload.file_paths),
            MILVUS_FIELD_VARCHAR_MAX_LENGTH,
        ),
        build_field_column(
            VECTOR_FIELD_START_LINE,
            DataType::Int64,
            ValueVec::Long(payload.start_lines),
            0,
        ),
        build_field_column(
            VECTOR_FIELD_CONTENT,
            DataType::VarChar,
            ValueVec::String(payload.contents),
            MILVUS_METADATA_VARCHAR_MAX_LENGTH,
        ),
    ]
}

pub(super) fn parse_milvus_ids(
    result: &milvus::proto::milvus::MutationResult,
) -> Result<Vec<String>> {
    let ids = result
        .i_ds
        .as_ref()
        .ok_or_else(|| Error::vector_db("Milvus mutation result missing IDs field"))?;
    let id_field = ids
        .id_field
        .as_ref()
        .ok_or_else(|| Error::vector_db("Milvus mutation result has IDs but missing id_field"))?;
    match id_field {
        milvus::proto::schema::i_ds::IdField::IntId(int_ids) => {
            Ok(int_ids.data.iter().map(ToString::to_string).collect())
        }
        milvus::proto::schema::i_ds::IdField::StrId(str_ids) => Ok(str_ids.data.clone()),
    }
}
