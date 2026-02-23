//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use std::borrow::Cow;
use std::path::PathBuf;
use std::sync::OnceLock;

use serde::Serialize;

use super::context::{Context, ContextManager};

#[derive(Debug)]
pub(crate) struct TemplateInfo {
    pub(crate) path: Option<PathBuf>,
    pub(crate) engine_ext: &'static str,
    pub(crate) data_type: String,
}

/// A rendered template response for Axum, backed by `serde_json::Value`.
#[derive(Debug)]
pub struct Template {
    name: Cow<'static, str>,
    value: Result<serde_json::Value, serde_json::Error>,
}

impl Template {
    /// Creates a new [`Template`] by serializing `context` for the template `name`.
    #[inline]
    pub fn render<S, C>(name: S, context: C) -> Template
    where
        S: Into<Cow<'static, str>>,
        C: Serialize,
    {
        Template {
            name: name.into(),
            value: serde_json::to_value(context),
        }
    }

    #[inline(always)]
    pub(crate) fn finalize(self, ctxt: &Context) -> Result<(String, String), ()> {
        let name = &*self.name;
        let info = ctxt.templates.get(name).ok_or_else(|| {
            let ts: Vec<_> = ctxt
                .templates
                .keys()
                .map(std::string::String::as_str)
                .collect();
            let detail = format!(
                "'{}' not found. Known: {}. Root: {:?}",
                name,
                ts.join(", "),
                ctxt.root
            );
            mcb_domain::error!("Template", "Template does not exist", &detail);
        })?;

        let value = self.value.map_err(|e| {
            mcb_domain::error!("Template", "Context serialization failed", &e);
        })?;

        let string = ctxt.engines.render(name, info, value).ok_or_else(|| {
            mcb_domain::error!("Template", "Template failed to render", &name);
        })?;

        Ok((info.data_type.clone(), string))
    }
}

static AXUM_CONTEXT: OnceLock<ContextManager> = OnceLock::new();

/// # Panics
/// Panics if the template context cannot be initialized.
pub fn init_axum_context(
    template_dir: &str,
    callback: impl Fn(&mut super::engine::Engines) + Send + Sync + 'static,
) {
    use super::context::Context;

    if AXUM_CONTEXT.get().is_some() {
        return;
    }

    let root = PathBuf::from(template_dir);
    let boxed_cb: super::context::Callback = Box::new(move |engines| {
        callback(engines);
        Ok(())
    });
    let ctxt = Context::initialize(&root, &boxed_cb).unwrap_or_else(|| {
        unreachable!(
            "Template context initialization is infallible when template dir and callback are valid"
        );
    });
    let _ = AXUM_CONTEXT.set(ContextManager::new(ctxt));
}

impl axum::response::IntoResponse for Template {
    fn into_response(self) -> axum::response::Response {
        let Some(cm) = AXUM_CONTEXT.get() else {
            mcb_domain::error!(
                "Template",
                "Template rendered before init_axum_context() was called"
            );
            return (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Template engine not initialized",
            )
                .into_response();
        };

        match self.finalize(&cm.context()) {
            Ok((content_type, body)) => {
                ([(axum::http::header::CONTENT_TYPE, content_type)], body).into_response()
            }
            Err(()) => (
                axum::http::StatusCode::INTERNAL_SERVER_ERROR,
                "Template rendering failed",
            )
                .into_response(),
        }
    }
}
