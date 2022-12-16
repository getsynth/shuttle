use crossterm::style::Color;

use crate::helpers::{self, APPS_FQDN};

#[test]
fn hello_world_warp() {
    let client =
        helpers::Services::new_docker("hello-world (warp)", "warp/hello-world", Color::Cyan);
    client.deploy();

    let request_text = client
        .get("hello")
        .header("Host", format!("hello-world-warp-app.{}", *APPS_FQDN))
        .send()
        .unwrap()
        .text()
        .unwrap();

    assert_eq!(request_text, "Hello, World!");
}
