use std::error::Error;
use std::fs;

static OUT_DIR: &str = "src/proto";

fn main() -> Result<(), Box<dyn Error>> {
    let protos = ["src/proto/finalitygadget.proto"];

    fs::create_dir_all(OUT_DIR).unwrap();
    tonic_build::configure()
        // .include_file("mod.rs")
        .build_server(false)
        .build_client(true)
        .out_dir(OUT_DIR)
        // .file_descriptor_set_path("finalitygadget.rs") // Optional: save file descriptor
        // .file_descriptor_set_path("src/proto/finalitygadget.rs") // Optional: save file descriptor
        .compile_protos(&protos, &["src/proto"])?;
        // .compile_protos(&protos, &[""])?;

    rerun(&protos);

    Ok(())
}

fn rerun(proto_files: &[&str]) {
    for proto_file in proto_files {
        println!("cargo:rerun-if-changed={}", proto_file);
    }
}
