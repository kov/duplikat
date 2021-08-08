fn main() {
    gbuild::Resource::builder()
        .src_dir("src")
        .definition_file("src/duplikat.gresource.xml")
        .build()
        .expect("Bad arguments for gresource compilation")
        .compile();

    println!("cargo:rerun-if-changed=build.rs");
}
