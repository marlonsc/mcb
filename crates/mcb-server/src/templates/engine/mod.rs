use std::collections::HashMap;
use std::path::Path;

use handlebars::Handlebars;
use rocket::serde::Serialize;

use super::template::TemplateInfo;

mod handlebars_engine;

pub(crate) trait Engine: Send + Sync + Sized + 'static {
    const EXT: &'static str;

    fn init<'a>(templates: impl Iterator<Item = (&'a str, &'a Path)>) -> Option<Self>;
    fn render<C: Serialize>(&self, name: &str, context: C) -> Option<String>;
}

/// A structure exposing access to the Handlebars templating engine.
///
/// When calling methods on the `Handlebars` instance, ensure you use types
/// imported from the `handlebars` crate to avoid version mismatches.
pub struct Engines {
    /// The Handlebars templating engine.
    pub handlebars: Handlebars<'static>,
}

impl Engines {
    pub(crate) const ENABLED_EXTENSIONS: &'static [&'static str] = &[Handlebars::<'static>::EXT];

    pub(crate) fn init(templates: &HashMap<String, TemplateInfo>) -> Option<Engines> {
        let named_templates = templates
            .iter()
            .filter(|&(_, i)| i.engine_ext == Handlebars::<'static>::EXT)
            .filter_map(|(k, i)| Some((k.as_str(), i.path.as_ref()?)))
            .map(|(k, p)| (k, p.as_path()));

        let handlebars = <Handlebars<'static> as Engine>::init(named_templates)?;
        Some(Engines { handlebars })
    }

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

    /// Returns iterator over template (name, engine_extension).
    pub(crate) fn templates(&self) -> impl Iterator<Item = (&str, &'static str)> {
        self.handlebars
            .get_templates()
            .keys()
            .map(|name| (name.as_str(), Handlebars::<'static>::EXT))
    }
}
