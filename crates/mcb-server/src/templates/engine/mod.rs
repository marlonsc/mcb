//! Template engine abstraction and management.
//!
//! This module defines the [`Engine`] trait for templating engines and the
//! [`Engines`] struct which aggregates and manages enabled engine instances.

use std::collections::HashMap;
use std::path::Path;

use handlebars::Handlebars;
use rocket::serde::Serialize;

use super::embedded;
use super::template::TemplateInfo;

mod handlebars_engine;

/// Trait for templating engine implementations.
///
/// Each supported templating engine must implement this trait to be used
/// by the server's templating system.
pub(crate) trait Engine: Send + Sync + Sized + 'static {
    /// The file extension associated with this engine (e.g., "hbs").
    const EXT: &'static str;

    /// Initializes the engine with a set of template names and their paths.
    fn init<'a>(templates: impl Iterator<Item = (&'a str, &'a Path)>) -> Option<Self>;

    /// Renders a template with the given name and context.
    ///
    /// # Errors
    ///
    /// Returns `None` if rendering fails or the template is not found.
    fn render<C: Serialize>(&self, name: &str, context: C) -> Option<String>;
}

/// Manages template engine instances.
pub struct Engines {
    /// The Handlebars templating engine.
    pub handlebars: Handlebars<'static>,
}

impl Engines {
    /// Extensions currently supported by the server.
    pub(crate) const ENABLED_EXTENSIONS: &'static [&'static str] = &[Handlebars::<'static>::EXT];

    /// Initializes all enabled engines with the provided templates.
    pub(crate) fn init(templates: &HashMap<String, TemplateInfo>) -> Option<Engines> {
        let named_templates = templates
            .iter()
            .filter(|&(_, i)| i.engine_ext == Handlebars::<'static>::EXT)
            .filter_map(|(k, i)| Some((k.as_str(), i.path.as_ref()?)))
            .map(|(k, p)| (k, p.as_path()));

        let handlebars = <Handlebars<'static> as Engine>::init(named_templates)?;
        Some(Engines { handlebars })
    }

    /// Initializes engines using only embedded templates.
    pub(crate) fn init_embedded() -> Option<Engines> {
        let mut hb = Handlebars::new();
        if embedded::register_embedded(&mut hb) {
            Some(Engines { handlebars: hb })
        } else {
            None
        }
    }

    /// Renders a template using the engine specified in `info`.
    pub(crate) fn render<C: Serialize>(
        &self,
        name: &str,
        info: &TemplateInfo,
        context: C,
    ) -> Option<String> {
        if info.engine_ext == Handlebars::<'static>::EXT {
            return Engine::render(&self.handlebars, name, context);
        }

        None
    }

    /// Returns an iterator over all registered templates and their engine extensions.
    pub(crate) fn templates(&self) -> impl Iterator<Item = (&str, &'static str)> {
        self.handlebars
            .get_templates()
            .keys()
            .map(|name| (name.as_str(), Handlebars::<'static>::EXT))
    }
}
