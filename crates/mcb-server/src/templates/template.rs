use std::borrow::Cow;
use std::path::PathBuf;

use rocket::fairing::Fairing;
use rocket::figment::{error::Error, value::Value};
use rocket::http::{ContentType, Status};
use rocket::request::Request;
use rocket::response::{self, Responder};
use rocket::serde::Serialize;
use rocket::yansi::Paint;
use rocket::{Ignite, Orbit, Rocket, Sentinel};

use super::context::{Context, ContextManager};
use super::engine::Engines;
use super::fairing::TemplateFairing;

pub(crate) const DEFAULT_TEMPLATE_DIR: &str = "templates";

#[derive(Debug)]
pub(crate) struct TemplateInfo {
    pub(crate) path: Option<PathBuf>,
    pub(crate) engine_ext: &'static str,
    pub(crate) data_type: ContentType,
}

/// Responder that renders a dynamic template.
///
/// `Template` serves as a _proxy_ type for rendering a template and _does not_
/// contain the rendered template itself. The template is lazily rendered, at
/// response time. To render a template greedily, use [`Template::show()`].
#[derive(Debug)]
pub struct Template {
    name: Cow<'static, str>,
    value: Result<Value, Error>,
}

impl Template {
    /// Returns a fairing that initializes and maintains templating state.
    ///
    /// This fairing, or the one returned by [`Template::custom()`], _must_ be
    /// attached to any `Rocket` instance that wishes to render templates.
    pub fn fairing() -> impl Fairing {
        Template::custom(|_| {})
    }

    /// Returns a fairing that initializes and maintains templating state.
    ///
    /// Unlike [`Template::fairing()`], this method allows you to configure
    /// templating engines via the function `f`.
    pub fn custom<F>(f: F) -> impl Fairing
    where
        F: Send + Sync + 'static + Fn(&mut Engines),
    {
        Self::try_custom(move |engines| {
            f(engines);
            Ok(())
        })
    }

    /// Returns a fairing that initializes and maintains templating state.
    ///
    /// This variant of [`Template::custom()`] allows a fallible `f`.
    pub fn try_custom<F>(f: F) -> impl Fairing
    where
        F: Send + Sync + 'static + Fn(&mut Engines) -> Result<(), Box<dyn std::error::Error>>,
    {
        TemplateFairing {
            callback: Box::new(f),
        }
    }

    /// Render the template named `name` with the context `context`.
    #[inline]
    pub fn render<S, C>(name: S, context: C) -> Template
    where
        S: Into<Cow<'static, str>>,
        C: Serialize,
    {
        Template {
            name: name.into(),
            value: Value::serialize(context),
        }
    }

    /// Render the template named `name` with the context `context` into a
    /// `String`. This method should only be used during testing.
    #[inline]
    pub fn show<S, C>(rocket: &Rocket<Orbit>, name: S, context: C) -> Option<String>
    where
        S: Into<Cow<'static, str>>,
        C: Serialize,
    {
        let ctxt = rocket
            .state::<ContextManager>()
            .map(ContextManager::context)
            .or_else(|| {
                warn!("Uninitialized template context: missing fairing.");
                info!("To use templates, you must attach `Template::fairing()`.");
                None
            })?;

        Template::render(name, context)
            .finalize(&ctxt)
            .ok()
            .map(|v| v.1)
    }

    #[inline(always)]
    pub(crate) fn finalize(self, ctxt: &Context) -> Result<(ContentType, String), Status> {
        let name = &*self.name;
        let info = ctxt.templates.get(name).ok_or_else(|| {
            let ts: Vec<_> = ctxt.templates.keys().map(|s| s.as_str()).collect();
            error_!("Template '{}' does not exist.", name);
            info_!("Known templates: {}.", ts.join(", "));
            info_!("Searched in {:?}.", ctxt.root);
            Status::InternalServerError
        })?;

        let value = self.value.map_err(|e| {
            error_!("Template context failed to serialize: {}.", e);
            Status::InternalServerError
        })?;

        let string = ctxt.engines.render(name, info, value).ok_or_else(|| {
            error_!("Template '{}' failed to render.", name);
            Status::InternalServerError
        })?;

        Ok((info.data_type.clone(), string))
    }
}

impl<'r> Responder<'r, 'static> for Template {
    fn respond_to(self, req: &'r Request<'_>) -> response::Result<'static> {
        let ctxt = req.rocket().state::<ContextManager>().ok_or_else(|| {
            error_!("Uninitialized template context: missing fairing.");
            info_!("To use templates, you must attach `Template::fairing()`.");
            Status::InternalServerError
        })?;

        self.finalize(&ctxt.context())?.respond_to(req)
    }
}

impl Sentinel for Template {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        if rocket.state::<ContextManager>().is_none() {
            let template = "Template".primary().bold();
            let fairing = "Template::fairing()".primary().bold();
            error!(
                "returning `{}` responder without attaching `{}`.",
                template, fairing
            );
            info_!("To use or query templates, you must attach `{}`.", fairing);
            return true;
        }

        false
    }
}

/// A macro to easily create a template rendering context.
///
/// Invocations of this macro expand to a value of an anonymous type which
/// implements [`Serialize`]. Fields can be literal expressions or
/// variables captured from a surrounding scope, as long as all fields implement
/// `Serialize`.
#[macro_export]
macro_rules! context {
    ($($key:ident $(: $value:expr)?),*$(,)?) => {{
        use ::rocket::serde::ser::{Serialize, Serializer, SerializeMap};
        use ::std::fmt::{Debug, Formatter};
        use ::std::result::Result;

                struct ContextMacroCtxObject<$($key: Serialize),*> {
            $($key: $key),*
        }

                impl<$($key: Serialize),*> Serialize for ContextMacroCtxObject<$($key),*> {
            fn serialize<S>(&self, serializer: S) -> Result<S::Ok, S::Error>
                where S: Serializer,
            {
                let mut map = serializer.serialize_map(None)?;
                $(map.serialize_entry(stringify!($key), &self.$key)?;)*
                map.end()
            }
        }

                impl<$($key: Debug + Serialize),*> Debug for ContextMacroCtxObject<$($key),*> {
            fn fmt(&self, f: &mut Formatter<'_>) -> ::std::fmt::Result {
                f.debug_struct("context!")
                    $(.field(stringify!($key), &self.$key))*
                    .finish()
            }
        }

        ContextMacroCtxObject {
            $($key $(: $value)?),*
        }
    }};
}
