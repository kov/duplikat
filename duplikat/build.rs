use std::env;
use std::path::Path;
use std::io::{Write, BufWriter};
use std::fs::File;
use log::info;

fn main() {
    gbuild::Resource::builder()
        .src_dir("src")
        .definition_file("src/duplikat.gresource.xml")
        .build()
        .expect("Bad arguments for gresource compilation")
        .compile();

    println!("cargo:rerun-if-changed=build.rs");

    // Add a global variable that sets the installation prefix.
    let default_prefix = "/usr/local".to_string();
    let prefix_str = match env::var("DUPLIKAT_PREFIX") {
        Ok(p) => p,
        Err(_) => {
            info!("Using default prefix {}", default_prefix);
            default_prefix
        },
    };

    let out_dir = env::var("OUT_DIR").unwrap();
    let dest_path = Path::new(&out_dir).join("prefix.rs");
    let mut prefix_file = BufWriter::new(File::create(&dest_path).unwrap());
    write!(prefix_file,
        "pub static PREFIX: &'static str = \"{}\";",
        prefix_str
    ).unwrap();

    println!("cargo:rerun-if-env-changed=DUPLIKAT_PREFIX")
}
