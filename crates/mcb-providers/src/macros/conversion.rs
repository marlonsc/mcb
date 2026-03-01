//! Conversion macros for SeaORM entities ↔ domain types.
//!
//! Replaces `scripts/codegen-conversions.py` and `config/conversions.toml`.
//!
//! # Supported field types
//!
//! | Keyword            | Model → Domain                              | Domain → ActiveModel                     |
//! |--------------------|---------------------------------------------|------------------------------------------|
//! | `direct`           | `m.field`                                   | `Set(e.field)`                           |
//! | `enum`             | `m.f.parse::<T>().unwrap_or(default)`       | `Set(e.f.to_string())`                   |
//! | `enum_opt`         | `m.f.as_deref()…parse…unwrap_or(default)`   | `Set(Some(e.f.to_string()))`             |
//! | `bool`             | `m.f != 0`                                  | `Set(i64::from(e.f))`                    |
//! | `bool_opt`         | `m.f.is_some_and(\|v\| v != 0)`             | `Set(Some(i64::from(e.f)))`              |
//! | `opt_bool`         | `m.f.map(\|v\| v != 0)`                     | `Set(e.f.map(i64::from))`               |
//! | `int_cast`         | `m.f as i32`                                | `Set(i64::from(e.f))`                    |
//! | `opt_int_cast`     | `m.f.map(\|v\| v as i32)`                   | `Set(e.f.map(i64::from))`               |
//! | `json_array`       | `from_str(m.f?)…unwrap_or_default()`        | `Set(Some(to_string(&e.f)…))`            |
//! | `json_array_req`   | `from_str(&m.f)…unwrap_or_default()`        | `Set(to_string(&e.f)…)`                 |
//! | `json_object`      | `from_str::<T>(m.f?)…unwrap_or_default()`   | `Set(Some(to_string(&e.f)…))`            |
//! | `json_object_opt`  | `from_str::<T>(m.f?)…`                      | `Set(e.f.as_ref()…to_string…ok())`       |
//! | `json_value`       | `from_str(&m.f)…unwrap_or(Null)`            | `Set(to_string(&e.f)…)`                 |
//! | `unwrap_default`   | `m.f.unwrap_or_default()`                   | `Set(Some(e.f))`                         |
//! | `computed`         | custom expression using `m`                 | *(field skipped in ActiveModel)*         |
//! | `not_set`          | *(field skipped in Model)*                  | `NotSet`                                 |

/// Generate `From<entity::Model> for DomainType` and
/// `From<DomainType> for entity::ActiveModel` implementations.
///
/// # Examples
///
/// Simple entity (all direct fields):
/// ```rust,ignore
/// impl_conversion!(organization, Organization,
///     direct: [id, name, slug, settings_json, created_at, updated_at]
/// );
/// ```
///
/// Entity with enums:
/// ```rust,ignore
/// impl_conversion!(plan, Plan,
///     direct: [id, org_id, project_id, title, description, created_by, created_at, updated_at],
///     enums: { status: PlanStatus = PlanStatus::Draft }
/// );
/// ```
///
/// Complex entity (multiple converter types):
/// ```rust,ignore
/// impl_conversion!(observation, Observation,
///     direct: [id, project_id, content, content_hash, created_at, embedding_id],
///     json_arrays: [tags],
///     enum_opts: { observation_type as r#type: ObservationType = ObservationType::Context },
///     json_objects: { metadata: ObservationMetadata }
/// );
/// ```
#[macro_export]
macro_rules! impl_conversion {
    (
        $entity_mod:ident, $DomainType:ty
        // All sections are optional; order is fixed.
        $(, direct: [$($d:ident),* $(,)?] )?
        $(, enums: { $($e_field:ident : $EType:ty = $e_default:expr),* $(,)? } )?
        $(, enum_opts: { $($eo_db:ident as $eo_domain:tt : $EOType:ty = $eo_default:expr),* $(,)? } )?
        $(, bools: [$($b_field:ident),* $(,)?] )?
        $(, bool_opts: [$($bo_field:ident),* $(,)?] )?
        $(, opt_bools: [$($ob_field:ident),* $(,)?] )?
        $(, int_casts: [$($ic_field:ident),* $(,)?] )?
        $(, opt_int_casts: [$($oic_field:ident),* $(,)?] )?
        $(, json_arrays: [$($ja_field:ident),* $(,)?] )?
        $(, json_arrays_req: [$($jar_field:ident),* $(,)?] )?
        $(, json_objects: { $($jo_field:ident : $JOType:ty),* $(,)? } )?
        $(, json_objects_opt: { $($joo_field:ident : $JOOType:ty),* $(,)? } )?
        $(, json_values: [$($jv_field:ident),* $(,)?] )?
        $(, unwrap_defaults: [$($ud_field:ident),* $(,)?] )?
        $(, computed: { $($cf_domain:tt = $cf_expr:expr),* $(,)? } )?
        $(, not_set: [$($ns_field:ident),* $(,)?] )?
        $(,)?
    ) => {
        impl From<$entity_mod::Model> for $DomainType {
            fn from(m: $entity_mod::Model) -> Self {
                Self {
                    // direct
                    $($( $d: m.$d, )*)?

                    // enums
                    $($( $e_field: m.$e_field.parse::<$EType>().unwrap_or($e_default), )*)?

                    // enum_opts (db_field as domain_field)
                    $($( $eo_domain: m.$eo_db
                            .as_deref()
                            .and_then(|s| s.parse::<$EOType>().ok())
                            .unwrap_or($eo_default), )*)?

                    // bools
                    $($( $b_field: m.$b_field != 0, )*)?

                    // bool_opts
                    $($( $bo_field: m.$bo_field.is_some_and(|v| v != 0), )*)?

                    // opt_bools
                    $($( $ob_field: m.$ob_field.map(|v| v != 0), )*)?

                    // int_casts
                    $($( $ic_field: m.$ic_field as i32, )*)?

                    // opt_int_casts
                    $($( $oic_field: m.$oic_field.map(|v| v as i32), )*)?

                    // json_arrays
                    $($( $ja_field: m.$ja_field
                            .as_deref()
                            .and_then(|s| serde_json::from_str(s)
                                .map_err(|e| tracing::warn!(
                                    field = stringify!($ja_field),
                                    error = %e,
                                    "malformed JSON in DB column"
                                ))
                                .ok())
                            .unwrap_or_default(), )*)?

                    // json_arrays_req
                    $($( $jar_field: serde_json::from_str(&m.$jar_field)
                            .map_err(|e| tracing::warn!(
                                field = stringify!($jar_field),
                                error = %e,
                                "malformed JSON in DB column"
                            ))
                            .unwrap_or_default(), )*)?

                    // json_objects
                    $($( $jo_field: m.$jo_field
                            .as_deref()
                            .and_then(|s| serde_json::from_str::<$JOType>(s)
                                .map_err(|e| tracing::warn!(
                                    field = stringify!($jo_field),
                                    error = %e,
                                    "malformed JSON in DB column"
                                ))
                                .ok())
                            .unwrap_or_default(), )*)?

                    // json_objects_opt
                    $($( $joo_field: m.$joo_field
                            .as_deref()
                            .and_then(|s| serde_json::from_str::<$JOOType>(s)
                                .map_err(|e| tracing::warn!(
                                    field = stringify!($joo_field),
                                    error = %e,
                                    "malformed JSON in DB column"
                                ))
                                .ok()), )*)?

                    // json_values
                    $($( $jv_field: serde_json::from_str(&m.$jv_field)
                            .unwrap_or(serde_json::Value::Null), )*)?

                    // unwrap_defaults
                    $($( $ud_field: m.$ud_field.unwrap_or_default(), )*)?

                    // computed (custom expression, has access to `m`)
                    $($( $cf_domain: $cf_expr, )*)?
                }
            }
        }

        impl From<$DomainType> for $entity_mod::ActiveModel {
            fn from(e: $DomainType) -> Self {
                Self {
                    // direct
                    $($( $d: ActiveValue::Set(e.$d), )*)?

                    // enums
                    $($( $e_field: ActiveValue::Set(e.$e_field.to_string()), )*)?

                    // enum_opts (db_field as domain_field)
                    $($( $eo_db: ActiveValue::Set(Some(e.$eo_domain.to_string())), )*)?

                    // bools
                    $($( $b_field: ActiveValue::Set(i64::from(e.$b_field)), )*)?

                    // bool_opts
                    $($( $bo_field: ActiveValue::Set(Some(i64::from(e.$bo_field))), )*)?

                    // opt_bools
                    $($( $ob_field: ActiveValue::Set(e.$ob_field.map(i64::from)), )*)?

                    // int_casts
                    $($( $ic_field: ActiveValue::Set(i64::from(e.$ic_field)), )*)?

                    // opt_int_casts
                    $($( $oic_field: ActiveValue::Set(e.$oic_field.map(i64::from)), )*)?

                    // json_arrays
                    $($( $ja_field: ActiveValue::Set(Some(
                        serde_json::to_string(&e.$ja_field)
                            .unwrap_or_else(|_| "[]".into()),
                    )), )*)?

                    // json_arrays_req
                    $($( $jar_field: ActiveValue::Set(
                        serde_json::to_string(&e.$jar_field)
                            .unwrap_or_else(|_| "[]".into()),
                    ), )*)?

                    // json_objects
                    $($( $jo_field: ActiveValue::Set(Some(
                        serde_json::to_string(&e.$jo_field)
                            .unwrap_or_else(|_| "{}".into()),
                    )), )*)?

                    // json_objects_opt
                    $($( $joo_field: ActiveValue::Set(
                        e.$joo_field.as_ref()
                            .and_then(|v| serde_json::to_string(v).ok()),
                    ), )*)?

                    // json_values
                    $($( $jv_field: ActiveValue::Set(
                        serde_json::to_string(&e.$jv_field)
                            .unwrap_or_else(|_| "{}".into()),
                    ), )*)?

                    // unwrap_defaults
                    $($( $ud_field: ActiveValue::Set(Some(e.$ud_field)), )*)?

                    // not_set
                    $($( $ns_field: ActiveValue::NotSet, )*)?
                }
            }
        }
    };
}
