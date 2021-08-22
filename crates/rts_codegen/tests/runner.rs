use std::fs;
use std::env;
use std::path::Path;

use rts_codegen::gen_tokens;
use quote::quote;

#[test]
fn draw() {
    let shader_src = include_str!("draw.wgsl");
    let code = gen_tokens(shader_src, "draw.wgsl");

    // We need to modify the code to have a main method
    let code = quote! {
        mod shader {
            #code
        }
        fn main() {
            assert_eq!(shader::VsMain::NAME, "vs_main");
            assert_eq!(shader::FsMain::NAME, "fs_main");
            assert_eq!(shader::VsMain::STAGE, ::naga::ShaderStage::Vertex);
            assert_eq!(shader::FsMain::STAGE, ::naga::ShaderStage::Fragment);

            let locals = shader::Locals {
                transform: [
                    1.0, 0.0, 0.0, 0.0,
                    0.0, 1.0, 0.0, 0.0,
                    0.0, 0.0, 1.0, 0.0,
                    0.0, 0.0, 0.0, 1.0,
                ]
            };
        }
    };

    let root = Path::new(&env::var("CARGO_MANIFEST_DIR").unwrap()).join("tests/output");
    let path = root.join("draw_gen.rs");
    fs::write(&path, code.to_string()).unwrap();

    let module = naga::front::wgsl::parse_str(shader_src).unwrap();
    fs::write(&root.join("draw_gen.txt"), format!("{:#?}", &module)).unwrap();

    let t = trybuild::TestCases::new();
    t.pass(path);
}