//!
//! **Documentation**: [docs/modules/server.md](../../../../../docs/modules/server.md)
//!
use rocket::State;
use rocket::get;

use super::HttpTransportState;

#[get("/healthz")]
pub(super) fn healthz() -> &'static str {
    "OK"
}

#[get("/readyz")]
pub(super) fn readyz(_state: &State<HttpTransportState>) -> &'static str {
    "OK"
}
