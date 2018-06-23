extern crate bindgen;

fn main() {
    let out_file = if cfg!(feature = "idebuild") {
        let out_dir = std::env::current_dir().unwrap();
        let out_file = out_dir.join("src").join("linput").join("linputbindings.rs");
        out_file
    } else {
        let out_dir = std::path::PathBuf::from(std::env::var("OUT_DIR").unwrap());
        let out_file = out_dir.join("linux-input.rs");
        out_file
    };

    if !out_file.exists() {
        bindgen::Builder::default()
            .header("src/linput/linux-input.h")
            .generate()
            .expect("Unable to generate bindings")
            .write_to_file(out_file)
            .expect("Couldn't write bindings!");
    }
}
