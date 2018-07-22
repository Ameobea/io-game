extern crate protobuf_codegen_pure;

fn main() {
    // Generates Rust code from protocol buffer definitions
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/protos",
        input: &["../../schema/message.proto"],
        includes: &["../../schema"],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
    }).expect("protoc");
}
