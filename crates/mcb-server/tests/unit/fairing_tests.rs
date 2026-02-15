use mcb_server::templates::Template;
use rocket::fairing::{Fairing, Kind};

#[test]
fn template_fairing_exposes_expected_metadata() {
    let fairing = Template::fairing();
    let info = fairing.info();

    assert_eq!(info.name, "Templating");
    assert!(info.kind.is(Kind::Ignite));
    assert!(info.kind.is(Kind::Liftoff));
}
