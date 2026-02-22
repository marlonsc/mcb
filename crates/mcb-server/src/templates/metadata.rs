//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
use std::borrow::Cow;
use std::fmt;

use serde::Serialize;

use super::Template;
use super::context::ContextManager;

/// Provides read-only access to template engine metadata and rendering.
pub struct Metadata<'a>(&'a ContextManager);

impl Metadata<'_> {
    /// Returns `true` if a template with the given `name` is registered.
    #[must_use]
    pub fn contains_template(&self, name: &str) -> bool {
        self.0.context().templates.contains_key(name)
    }

    /// Returns `true` if the template engine is currently reloading (debug mode).
    #[must_use]
    pub fn reloading(&self) -> bool {
        self.0.is_reloading()
    }

    /// Renders the template `name` with the given `context`, returning `(content_type, body)`.
    pub fn render<S, C>(&self, name: S, context: C) -> Option<(String, String)>
    where
        S: Into<Cow<'static, str>>,
        C: Serialize,
    {
        Template::render(name.into(), context)
            .finalize(&self.0.context())
            .ok()
    }
}

impl fmt::Debug for Metadata<'_> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        f.debug_map().entries(&self.0.context().templates).finish()
    }
}
