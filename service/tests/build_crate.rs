use std::{io::Write, path::Path};

use shuttle_service::loader::build_crate;

struct DummyWriter {}

impl Write for DummyWriter {
    fn write(&mut self, buf: &[u8]) -> std::io::Result<usize> {
        Ok(buf.len())
    }

    fn flush(&mut self) -> std::io::Result<()> {
        Ok(())
    }
}

#[test]
fn not_shuttle() {
    let buf = Box::new(DummyWriter {});
    let project_path = format!("{}/tests/resources/not-shuttle", env!("CARGO_MANIFEST_DIR"));
    let so_path = build_crate(Path::new(&project_path), buf).unwrap();

    assert!(
        so_path
            .display()
            .to_string()
            .ends_with("tests/resources/not-shuttle/target/debug/libnot_shuttle.so"),
        "did not get expected so_path: {}",
        so_path.display()
    );
}

#[test]
#[should_panic(
    expected = "a cdylib was not created. Try adding the following to the Cargo.toml of the service:\n[lib]\ncrate-type = [\"cdylib\"]\n"
)]
fn not_cdylib() {
    let buf = Box::new(DummyWriter {});
    let project_path = format!("{}/tests/resources/not-cdylib", env!("CARGO_MANIFEST_DIR"));
    build_crate(Path::new(&project_path), buf).unwrap();
}

#[test]
#[should_panic(expected = "failed to read")]
fn not_found() {
    let buf = Box::new(DummyWriter {});
    let project_path = format!(
        "{}/tests/resources/non-existing",
        env!("CARGO_MANIFEST_DIR")
    );
    build_crate(Path::new(&project_path), buf).unwrap();
}
