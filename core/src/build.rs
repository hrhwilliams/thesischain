use std::{env, fs, path::PathBuf};

fn main() {
    let manifest_dir = PathBuf::from(env::var("CARGO_MANIFEST_DIR").expect("manifest dir"));
    let proto_dir = manifest_dir.join("proto");
    let out_dir = manifest_dir.join("src").join("proto_generated");

    fs::create_dir_all(&out_dir).expect("create proto output directory");

    prost_build::Config::new()
        .out_dir(&out_dir)
        .compile_protos(&[proto_dir.join("dchat.proto")], &[proto_dir])
        .expect("compile protobufs");
}
