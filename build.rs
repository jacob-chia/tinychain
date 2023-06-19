fn main() {
    prost_build::Config::new()
        .out_dir("src/network/schema")
        .compile_protos(&["api.v1.proto"], &["src/network/schema"])
        .unwrap();
}
