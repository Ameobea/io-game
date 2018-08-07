extern crate config_loader;
extern crate protobuf_codegen_pure;

fn main() {
    // Generates Rust code from protocol buffer definitions
    protobuf_codegen_pure::run(protobuf_codegen_pure::Args {
        out_dir: "src/protos",
        input: &[
            "../../schema/message_common.proto",
            "../../schema/server_messages.proto",
            "../../schema/client_messages.proto",
            "../../schema/channel_messages.proto",
        ],
        includes: &["../../schema"],
        customize: protobuf_codegen_pure::Customize {
            ..Default::default()
        },
    }).expect("Protobuf codegen error");

    println!("rerun-if-changed=../../schema");
    println!("rerun-if-changed=../../config");

    config_loader::build_config();
}
