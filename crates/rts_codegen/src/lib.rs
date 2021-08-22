use naga::{back::spv, valid::{self, Capabilities}};
use proc_macro2::TokenStream;
use quote::quote;
use syn::Token;

/// TODO: Use a Result with custom error.
pub fn gen_tokens(source: &str, file_name: &str) -> TokenStream {
    let module = naga::front::wgsl::parse_str(source).unwrap();
    let shaders = module
        .entry_points
        .iter()
        .map(|ep| {
            let stage = format!("{:?}", ep.stage).parse::<TokenStream>().unwrap();

            let name = &ep.name;
            let st_name = to_pascal_case(name).parse::<TokenStream>().unwrap();

            quote! {
                pub struct #st_name;
                impl #st_name {
                    pub const STAGE: ::naga::ShaderStage = ::naga::ShaderStage::#stage;
                    pub const NAME: &'static str = #name;
                }
            }
        })
        .collect::<TokenStream>();

    let disclaimer = format!("//! THIS CODE IS GENERATED FROM {:?}", file_name)
        .parse::<TokenStream>()
        .unwrap();

    let info = valid::Validator::new(valid::ValidationFlags::all(), Capabilities::empty())
        .validate(&module)
        .unwrap();
    let options = spv::Options::default();
    let raw_spirv = spv::write_vec(&module, &info, &options).unwrap();
    let spirv = raw_spirv
        .iter()
        .cloned()
        .collect::<syn::punctuated::Punctuated<u32, Token!(,)>>();

    let types = module.types.iter().filter_map(|(_, ty)| match (&ty.name, &ty.inner) {
        (Some(name), naga::TypeInner::Struct { members, .. }) => {
            let st_name = name.parse::<TokenStream>().unwrap();
            let st_members = members.iter().filter_map(|m| {
                // NOTE: Maybe map into a Result?
                let name = m.name.as_ref()?.parse::<TokenStream>().ok()?;
                match &module.types[m.ty] {
                    naga::Type {
                        name: Some(ty_name),
                        .. // NOTE: Not sure if I need to check the inner type in this case 
                    } => {
                        let ty_name = ty_name.parse::<TokenStream>().unwrap();
                        Some(quote! {
                            pub #name: #ty_name
                        })
                    }
                    naga::Type {
                        inner: naga::TypeInner::Vector {
                            size,
                            kind,
                            width,
                        },
                        ..
                    } => {
                        let element_type = match (kind, width) {
                            (naga::ScalarKind::Float, 2) => quote! { f16 },
                            (naga::ScalarKind::Float, 4) => quote! { f32 },
                            (naga::ScalarKind::Float, 8) => quote! { f64 },
                            _ => panic!("Unsupported type: {:?}, {:?}, {}", kind, size, width)
                        };
                        let num_elements = match size {
                            naga::VectorSize::Bi => 2usize,
                            naga::VectorSize::Tri => 3,
                            naga::VectorSize::Quad => 4,
                        };
                        Some(quote! {
                            pub #name: [#element_type; #num_elements]
                        })
                    }
                    naga::Type {
                        inner: naga::TypeInner::Matrix {
                            rows,
                            columns,
                            width,
                        },
                        ..
                    } => {
                        let element_type = match width {
                            2 => quote!{ f16 },
                            4 => quote!{ f32 },
                            8 => quote!{ f64 },
                            _ => panic!("Unsupported matrix format: {:?}, {:?}, {}", rows, columns, width)
                        };
                        let rows = match rows {
                            // NOTE: for uniform structs this may need to be 4
                            naga::VectorSize::Bi => 2usize,
                            // NOTE: this too
                            naga::VectorSize::Tri => 3,
                            naga::VectorSize::Quad => 4,
                        };
                        let columns = match columns {
                            naga::VectorSize::Bi => 2,
                            naga::VectorSize::Tri => 3,
                            naga::VectorSize::Quad => 4,
                        };
                        let num_elements = rows * columns;
                        Some(quote! {
                            pub #name: [#element_type; #num_elements]
                        })
                    }
                    _ => None
                }
            }).collect::<syn::punctuated::Punctuated<_, Token!(,)>>();

            Some(quote! {
                #[repr(C)]
                pub struct #st_name {
                    #st_members
                }
            })
        }
        _ => None
    }).collect::<TokenStream>();


    quote! {
        #![allow(dead_code)]
        #disclaimer
        #shaders
        #types

        // WIP Example
        // pub struct Group0 {
        //     bind_group_layout: ::wgpu::BindGroupLayout,
        // }

        // pub struct Group0Resources<'a> {
        //     color: &'a ::wgpu::TextureView,
        //     sampler: &'a ::wgpu::Sampler,
        // }

        // impl Group0 {
        //     pub fn new(device: &::wgpu::Device) -> Self {
        //         let bind_group_layout = device.create_bind_group_layout(&::wgpu::BindGroupLayoutDescriptor {
        //             label: Some("BindGroupLayout Group0"),
        //             entries: &[

        //             ]
        //         });
        //         Self { bind_group_layout }
        //     }

        //     pub fn bind(device: &::wgpu::Device, resources: Group0Resources) -> ::wgpu::BindGroup {

        //     }
        // }

        pub const SPIRV: &'static [u32] = &[#spirv];
    }
}

fn to_pascal_case(s: &str) -> String {
    let mut out = String::new();
    let mut should_be_uppercase = true;
    for c in s.chars() {
        if c.is_ascii_alphanumeric() {
            out.push(if c.is_ascii_lowercase() && should_be_uppercase {
                should_be_uppercase = false;
                c.to_ascii_uppercase()
            } else {
                c.to_ascii_lowercase()
            });
        } else if c == '_' {
            should_be_uppercase = true;
        } else {
            panic!("Encountered invalid character {}", c);
        }
    }
    out
}
