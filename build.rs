use std::process::Command;

fn main() {
    println!("cargo:rerun-if-changed=resources/");

    // Ensure that the resources directory exists
    let status = Command::new("glib-compile-resources")
        .arg("--target=resources.gresources")
        .arg("--sourcedir=resources")
        .arg("resources/resources.gresources.xml")
        .status()
        .expect("Failed to execute glib-compile-resources");

    if !status.success() {
        panic!("glib-compile-resources failed");
    }
    println!("Resources compiled!!");
}
