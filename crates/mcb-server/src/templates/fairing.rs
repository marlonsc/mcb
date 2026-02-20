//!
//! **Documentation**: [docs/modules/server.md](../../../../docs/modules/server.md)
//!
//! Rocket fairing for managing template contexts and lifecycles.

use rocket::fairing::{self, Fairing, Info, Kind};
use rocket::{Build, Orbit, Rocket};

use super::context::{Callback, Context, ContextManager};
use super::engine::Engines;
use super::template::DEFAULT_TEMPLATE_DIR;

/// A Rocket fairing that initializes and manages the template context.
///
/// This fairing is responsible for:
/// - Initializing the template `Context` during the ignition phase.
/// - Managing the `ContextManager` in the Rocket state.
/// - Printing initialization details during liftoff.
/// - Reloading templates on each request when debug assertions are enabled.
pub struct TemplateFairing {
    pub callback: Callback,
}

#[rocket::async_trait]
impl Fairing for TemplateFairing {
    fn info(&self) -> Info {
        let kind = Kind::Ignite | Kind::Liftoff;
        #[cfg(debug_assertions)]
        let kind = kind | Kind::Request;

        Info {
            kind,
            name: "Templating",
        }
    }

    async fn on_ignite(&self, rocket: Rocket<Build>) -> fairing::Result {
        use rocket::figment::value::magic::RelativePathBuf;

        let configured_dir = rocket
            .figment()
            .extract_inner::<RelativePathBuf>("template_dir")
            .map(|path| path.relative());

        let path = match configured_dir {
            Ok(dir) => dir,
            Err(e) if e.missing() => DEFAULT_TEMPLATE_DIR.into(),
            Err(e) => {
                rocket::config::pretty_print_error(e);
                return Err(rocket);
            }
        };

        if let Some(ctxt) = Context::initialize(&path, &self.callback) {
            Ok(rocket.manage(ContextManager::new(ctxt)))
        } else {
            error_!("Template initialization failed. Aborting launch.");
            Err(rocket)
        }
    }

    async fn on_liftoff(&self, rocket: &Rocket<Orbit>) {
        use rocket::{figment::Source, log::PaintExt, yansi::Paint};

        let Some(cm) = rocket.state::<ContextManager>() else {
            error_!("Template ContextManager missing in on_liftoff");
            return;
        };

        info!("{}{}:", "📐 ".emoji(), "Templating".magenta());
        info_!("directory: {}", Source::from(&*cm.context().root).primary());
        info_!("engines: {:?}", Engines::ENABLED_EXTENSIONS.primary());
    }

    #[cfg(debug_assertions)]
    async fn on_request(&self, req: &mut rocket::Request<'_>, _data: &mut rocket::Data<'_>) {
        let Some(cm) = req.rocket().state::<ContextManager>() else {
            error_!("Template ContextManager missing in on_request");
            return;
        };

        cm.reload_if_needed(&self.callback);
    }
}
