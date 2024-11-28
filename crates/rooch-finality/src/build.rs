use std::error::Error;
use std::fs;

static OUT_DIR: &str = "src/proto-gen";

fn main() -> Result<(), Box<dyn Error>> {
    let protos = ["proto/finalitygadget.proto"];

    fs::create_dir_all(OUT_DIR).unwrap();
    tonic_build::configure()
        // .build_server(true)
        .out_dir(OUT_DIR)
        .compile_protos(&protos, &["proto/"])?;
        // .compile_protos("proto/finalitygadget.proto")?;

    rerun(&protos);

    Ok(())
}

fn rerun(proto_files: &[&str]) {
    for proto_file in proto_files {
        println!("cargo:rerun-if-changed={}", proto_file);
    }
}
