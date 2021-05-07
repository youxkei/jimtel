use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, Data, Fields, DeriveInput};

#[proc_macro_derive(Params)]
pub fn derive_plugin_parameters(input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as DeriveInput);

    let ident = input.ident;

    (match input.data {
        Data::Struct(data) => {
            match data.fields {
                Fields::Named(fields) => {
                    let fields = fields.named.into_iter().enumerate().map(|(i, field)| (i as i32, field.ident.unwrap())).collect::<Vec<_>>();
                    let num_fields = fields.len();

                    let get_paramater_matches = fields.iter().map(|(i, field)| quote!{#i => self.#field.get()});
                    let set_paramater_matches = fields.iter().map(|(i, field)| quote!{#i => self.#field.set(value)});
                    let get_parameter_name_matches = fields.iter().map(|(i, field)| {
                        let field_name = format!("{}", field);
                        quote!{#i => #field_name.to_string()}
                    });

                    quote!{
                        impl #ident {
                            fn num_params() -> usize { #num_fields }
                        }

                        impl vst::plugin::PluginParameters for #ident {
                            fn get_parameter_name(&self, index: i32) -> String {
                                match index {
                                    #(#get_parameter_name_matches),*,
                                    _ => "".to_string(),
                                }
                            }

                            fn get_parameter(&self, index: i32) -> f32 {
                                match index {
                                    #(#get_paramater_matches),*,
                                    _ => 0.0,
                                }
                            }

                            fn set_parameter(&self, index: i32, value: f32) {
                                match index {
                                    #(#set_paramater_matches),*,
                                    _ => ()
                                }
                            }
                        }
                    }
                },

                _ => quote! {
                    compile_error!("struct required")
                },
            }
        },

        _ => quote! {
            compile_error!("struct required")
        },
    })
    .into()
}
