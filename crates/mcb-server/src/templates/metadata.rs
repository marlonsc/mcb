use std::borrow::Cow;
use std::fmt;

use rocket::http::{ContentType, Status};
use rocket::request::{self, FromRequest};
use rocket::serde::Serialize;
use rocket::yansi::Paint;
use rocket::{Ignite, Request, Rocket, Sentinel};

use super::Template;
use super::context::ContextManager;

/// Request guard for dynamically querying template metadata.
pub struct Metadata<'a>(&'a ContextManager);

impl Metadata<'_> {
    /// Returns `true` if the template with the given `name` is currently loaded.
    #[must_use]
    pub fn contains_template(&self, name: &str) -> bool {
        self.0.context().templates.contains_key(name)
    }

    /// Returns `true` if template reloading is enabled.
    #[must_use]
    pub fn reloading(&self) -> bool {
        self.0.is_reloading()
    }

    /// Render the template `name` with `context` into a `(ContentType, String)`.
    pub fn render<S, C>(&self, name: S, context: C) -> Option<(ContentType, String)>
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

impl Sentinel for Metadata<'_> {
    fn abort(rocket: &Rocket<Ignite>) -> bool {
        if rocket.state::<ContextManager>().is_none() {
            let md = "Metadata".primary().bold();
            let fairing = "Template::fairing()".primary().bold();
            error!("requested `{}` guard without attaching `{}`.", md, fairing);
            info_!("To use or query templates, you must attach `{}`.", fairing);
            return true;
        }

        false
    }
}

#[rocket::async_trait]
impl<'r> FromRequest<'r> for Metadata<'r> {
    type Error = ();

    async fn from_request(request: &'r Request<'_>) -> request::Outcome<Self, ()> {
        request.rocket().state::<ContextManager>().map_or_else(
            || {
                error_!("Uninitialized template context: missing fairing.");
                info_!("To use templates, you must attach `Template::fairing()`.");
                request::Outcome::Error((Status::InternalServerError, ()))
            },
            |cm| request::Outcome::Success(Metadata(cm)),
        )
    }
}
