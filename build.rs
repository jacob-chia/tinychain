fn main() {
    prost_build::Config::new()
        .out_dir("src/schema")
        .compile_protos(&["api.v1.proto"], &["src/schema"])
        .unwrap();
}
