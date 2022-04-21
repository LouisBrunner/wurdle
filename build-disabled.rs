use std::env;
use std::ops::Not;
use std::path::Path;
use std::process::Command;

const OPENAPI_GENERATOR: &str = "openapi-generator";
const OPENAPI_FILE: &str = "api/openapi.yaml";

fn main() {
    println!("cargo:rerun-if-changed={}", OPENAPI_FILE);
    println!("cargo:rerun-if-changed=build.rs");

    Command::new("which")
        .args(&[OPENAPI_GENERATOR])
        .status()
        .expect("failed to start `which`")
        .success()
        .not()
        .then(|| {
            panic!(
                "`{}` is missing, please install it separately",
                OPENAPI_GENERATOR
            )
        });

    Command::new(OPENAPI_GENERATOR)
        .args(&[
            "generate",
            "-i",
            OPENAPI_FILE,
            "-g",
            "rust-server",
            "-o",
            "openapi",
        ])
        .status()
        .expect("failed to start generator")
        .success()
        .not()
        .then(|| panic!("`{}` failed", OPENAPI_GENERATOR));
}
