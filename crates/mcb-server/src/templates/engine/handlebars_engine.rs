//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use std::path::Path;

use handlebars::Handlebars;
use serde::Serialize;

use super::Engine;

impl Engine for Handlebars<'static> {
    const EXT: &'static str = "hbs";

    fn init<'a>(templates: impl Iterator<Item = (&'a str, &'a Path)>) -> Option<Self> {
        let mut hb = Handlebars::new();
        let mut ok = true;
        for (name, path) in templates {
            if let Err(e) = hb.register_template_file(name, path) {
                let detail = format!("{name}: {e} (path: {})", path.display());
                mcb_domain::error!("Template", "Handlebars failed to register", &detail);
                ok = false;
            }
        }

        ok.then_some(hb)
    }

    fn render<C: Serialize>(&self, name: &str, context: C) -> Option<String> {
        if self.get_template(name).is_none() {
            mcb_domain::error!("Template", "Handlebars template does not exist", &name);
            return None;
        }

        Handlebars::render(self, name, &context)
            .map_err(|e| mcb_domain::error!("Template", "Handlebars render error", &e))
            .ok()
    }
}
