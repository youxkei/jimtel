use darling::{ast::Data, FromDeriveInput, FromField, FromMeta};
use proc_macro::TokenStream;
use quote::quote;
use syn::{parse_macro_input, DeriveInput, Ident};

#[derive(FromMeta)]
enum Unit {
    None,
    Ms,
    #[darling(rename = "dB")]
    Db,
    #[darling(rename = "dBFS")]
    Dbfs,
    #[darling(rename = "LKFS")]
    Lkfs,
}

impl Default for Unit {
    fn default() -> Self {
        Unit::None
    }
}

#[derive(FromDeriveInput)]
#[darling(supports(struct_named))]
struct Input {
    ident: Ident,

    data: Data<(), Field>,
}

#[derive(FromField)]
#[darling(attributes(param))]
struct Field {
    ident: Option<Ident>,

    #[darling(default)]
    unit: Unit,

    min: f32,
    max: f32,
}

#[proc_macro_derive(Params, attributes(param))]
pub fn derive_plugin_parameters(input: TokenStream) -> TokenStream {
    let input = Input::from_derive_input(&parse_macro_input!(input as DeriveInput)).unwrap();

    let ident = input.ident;

    let fields = input
        .data
        .take_struct()
        .unwrap()
        .fields
        .into_iter()
        .enumerate()
        .map(|(i, field)| (i as i32, field))
        .collect::<Vec<_>>();

    let num_fields = fields.len();

    let get_parameter_label_matches = fields.iter().map(|(i, field)| {
        let unit = match field.unit {
            Unit::None => "",
            Unit::Ms => "ms",
            Unit::Db => "dB",
            Unit::Dbfs => "dBFS",
            Unit::Lkfs => "LKFS",
        };

        quote! { #i => #unit.to_string() }
    });

    let get_parameter_text_matches = fields.iter().map(|(i, field)| {
        let ident = field.ident.as_ref().unwrap();
        let min = field.min;
        let width = field.max - field.min;

        match field.unit {
            Unit::Db | Unit::Dbfs | Unit::Lkfs => {
                quote! { #i => ((100.0 * (20.0 * self.#ident.get().log10())).round() / 100.0).to_string() }
            }

            _ => {
                quote! { #i => ((100.0 * self.#ident.get()).round() / 100.0).to_string() }
            }
        }
    });

    let get_parameter_name_matches = fields.iter().map(|(i, field)| {
        let field_name = format!("{}", field.ident.as_ref().unwrap());
        quote! {#i => #field_name.to_string()}
    });

    let get_paramater_matches = fields.iter().map(|(i, field)| {
        let ident = field.ident.as_ref().unwrap();
        let min = field.min;
        let width = field.max - field.min;

        match field.unit {
            Unit::Db | Unit::Dbfs | Unit::Lkfs => {
                quote! { #i => (20.0 * self.#ident.get().log10() - #min) / #width }
            }

            _ => {
                quote! { #i => (self.#ident.get() - #min) / #width }
            }
        }
    });

    let set_paramater_matches = fields.iter().map(|(i, field)| {
        let ident = field.ident.as_ref().unwrap();
        let min = field.min;
        let width = field.max - field.min;

        match field.unit {
            Unit::Db | Unit::Dbfs | Unit::Lkfs => {
                quote! { #i => self.#ident.set(10f32.powf((#min + #width * value) * 0.05)) }
            }

            _ => {
                quote! { #i => self.#ident.set(#min + #width * value) }
            }
        }
    });

    (quote! {
        impl #ident {
            fn num_params() -> usize { #num_fields }
        }

        impl vst::plugin::PluginParameters for #ident {
            fn get_parameter_label(&self, index: i32) -> String {
                match index {
                    #(#get_parameter_label_matches),*,
                    _ => "".to_string(),
                }
            }

            fn get_parameter_text(&self, index: i32) -> String {
                match index {
                    #(#get_parameter_text_matches),*,
                    _ => "".to_string(),
                }
            }

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
    })
    .into()
}
