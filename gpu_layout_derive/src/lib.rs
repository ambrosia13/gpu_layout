use proc_macro::TokenStream;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

#[proc_macro_derive(AsGpuBytes)]
pub fn gpu_bytes_derive(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);
    let name = input.ident;

    match input.data {
        syn::Data::Struct(data_struct) => {
            let write_calls = data_struct.fields.into_iter().map(|field| {
                let field_name = field.ident;

                quote! {
                    L::write(&mut buf, &self.#field_name);
                }
            });

            quote! {
                impl gpu_layout::AsGpuBytes for #name {
                    fn as_gpu_bytes<L: gpu_layout::GpuLayout + ?Sized>(&self) -> gpu_layout::GpuBytes {
                        let mut buf = gpu_layout::GpuBytes::empty();

                        #(
                            #write_calls
                        )*

                        buf
                    }
                }
            }.into()
        }
        _ => syn::Error::new_spanned(name, "AsGpuBytes cannot be derived for enums and unions")
            .to_compile_error()
            .into(),
    }
}
