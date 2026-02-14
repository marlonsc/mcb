macro_rules! tool_schema {
    ($(#[$meta:meta])* $vis:vis struct $name:ident { $($fields:tt)* }) => {
        $(#[$meta])*
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Deserialize, JsonSchema, Validate)]
        $vis struct $name {
            $($fields)*
        }
    };
}

macro_rules! tool_enum {
    ($(#[$meta:meta])* $vis:vis enum $name:ident { $($variants:tt)* }) => {
        $(#[$meta])*
        #[allow(missing_docs)]
        #[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, JsonSchema)]
        #[serde(rename_all = "snake_case")]
        $vis enum $name {
            $($variants)*
        }
    };
}

pub(crate) use tool_enum;
pub(crate) use tool_schema;
