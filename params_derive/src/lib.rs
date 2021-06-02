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
    Button,
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

    let get_name_matches = fields.iter().map(|(i, field)| {
        let name = format!("{}", field.ident.as_ref().unwrap());
        quote! { #i => #name.to_string() }
    });

    let get_unit_matches = fields.iter().map(|(i, field)| {
        let unit = match field.unit {
            Unit::None => "",
            Unit::Ms => "ms",
            Unit::Db => "dB",
            Unit::Dbfs => "dBFS",
            Unit::Lkfs => "LKFS",
            Unit::Button => "",
        };

        quote! { #i => #unit.to_string() }
    });

    let is_button_matches = fields.iter().map(|(i, field)| {
        let is_button = match field.unit {
            Unit::Button => true,
            _ => false,
        };

        quote! { #i => #is_button }
    });

    let get_range_matches = fields.iter().map(|(i, field)| {
        let Field { min, max, .. } = field;
        quote! { #i => #min..=#max }
    });

    let get_value_matches = fields.iter().map(|(i, field)| {
        let ident = field.ident.as_ref().unwrap();

        match field.unit {
            Unit::Db | Unit::Dbfs | Unit::Lkfs => {
                quote! { #i => 20.0 * self.#ident.get().log10() }
            }

            _ => {
                quote! { #i => self.#ident.get() }
            }
        }
    });

    let get_value_text_matches = fields.iter().map(|(i, _field)| {
        quote! { #i => ((10.0 * self.get_value(#i)).round() / 10.0).to_string() }
    });

    let set_value_matches = fields.iter().map(|(i, field)| {
        let ident = field.ident.as_ref().unwrap();

        match field.unit {
            Unit::Db | Unit::Dbfs | Unit::Lkfs => {
                quote! { #i => self.#ident.set(10f32.powf((value) * 0.05)) }
            }

            _ => {
                quote! { #i => self.#ident.set(value) }
            }
        }
    });

    let get_parameter_label_matches = fields.iter().map(|(i, _field)| {
        quote! { #i => self.get_unit(#i) }
    });

    let get_parameter_text_matches = fields.iter().map(|(i, _field)| {
        quote! { #i => self.get_value_text(#i) }
    });

    let get_parameter_name_matches = fields.iter().map(|(i, _field)| {
        quote! { #i => self.get_name(#i) }
    });

    let get_paramater_matches = fields.iter().map(|(i, field)| {
        let min = field.min;
        let width = field.max - field.min;

        quote! { #i => (self.get_value(#i) - #min) / #width }
    });

    let set_paramater_matches = fields.iter().map(|(i, field)| {
        let min = field.min;
        let width = field.max - field.min;

        quote! { #i => self.set_value(#i, #min + #width * value)}
    });

    (quote! {
        impl jimtel::params::Params for #ident {
            fn num_params() -> usize { #num_fields }

            fn index_range() -> std::ops::Range<i32> {
                0i32..(#num_fields as i32)
            }

            fn get_name(&self, index: i32) -> String {
                match index {
                    #(#get_name_matches),*,
                    _ => panic!(),
                }
            }

            fn get_unit(&self, index: i32) -> String {
                match index {
                    #(#get_unit_matches),*,
                    _ => panic!(),
                }
            }

            fn is_button(&self, index: i32) -> bool {
                match index {
                    #(#is_button_matches),*,
                    _ => panic!(),
                }
            }

            fn get_range(&self, index: i32) -> std::ops::RangeInclusive<f32> {
                match index {
                    #(#get_range_matches),*,
                    _ => panic!(),
                }
            }

            fn get_value(&self, index: i32) -> f32 {
                match index {
                    #(#get_value_matches),*,
                    _ => panic!(),
                }
            }

            fn get_value_text(&self, index: i32) -> String {
                match index {
                    #(#get_value_text_matches),*,
                    _ => panic!(),
                }
            }

            fn set_value(&self, index: i32, value: f32) {
                match index {
                    #(#set_value_matches),*,
                    _ => panic!(),
                }
            }
        }

        impl vst::plugin::PluginParameters for #ident {
            fn get_parameter_label(&self, index: i32) -> String {
                use jimtel::params::Params;

                match index {
                    #(#get_parameter_label_matches),*,
                    _ => panic!(),
                }
            }

            fn get_parameter_text(&self, index: i32) -> String {
                use jimtel::params::Params;

                match index {
                    #(#get_parameter_text_matches),*,
                    _ => panic!(),
                }
            }

            fn get_parameter_name(&self, index: i32) -> String {
                use jimtel::params::Params;

                match index {
                    #(#get_parameter_name_matches),*,
                    _ => panic!(),
                }
            }

            fn get_parameter(&self, index: i32) -> f32 {
                use jimtel::params::Params;

                match index {
                    #(#get_paramater_matches),*,
                    _ => panic!(),
                }
            }

            fn set_parameter(&self, index: i32, value: f32) {
                use jimtel::params::Params;

                match index {
                    #(#set_paramater_matches),*,
                    _ => panic!()
                }
            }

            fn string_to_parameter(&self, index: i32, text: String) -> bool {
                use jimtel::params::Params;

                match text.parse::<f32>() {
                    Ok(value) => {
                        self.set_value(index, value);
                        true
                    }

                    Err(_) => false
                }
            }

            fn get_bank_data(&self) -> Vec<u8> {
                use jimtel::params::Params;

                let mut vec = vec![];

                for index in Self::index_range() {
                    vec.push(self.get_value(index));
                }

                rmp_serde::to_vec(&vec).unwrap()
            }

            fn load_bank_data(&self, data: &[u8]) {
                use jimtel::params::Params;

                let vec: Vec<f32> = rmp_serde::from_read_ref(data).unwrap();

                for (index, value) in Self::index_range().zip(vec.into_iter()) {
                    self.set_value(index, value);
                }
            }
        }
    })
    .into()
}
