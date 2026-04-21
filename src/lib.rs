use std::iter::{Once, Repeat, repeat};

use proc_macro::TokenStream as Pm1TokenStream;
use proc_macro2::{Span, TokenStream as Pm2TokenStream};

use quote::quote;
use syn::Ident;

#[proc_macro_attribute]
pub fn builder(attr: Pm1TokenStream, item: Pm1TokenStream) -> Pm1TokenStream {
    builder_impl(attr.into(), item.into()).into()
}

fn builder_impl(attr: Pm2TokenStream, input: Pm2TokenStream) -> Pm2TokenStream {
    let input: syn::ItemStruct = match syn::parse2::<syn::ItemStruct>(input) {
        Ok(is) => is,
        Err(e) => {
            return darling::Error::from(e).write_errors();
        }
    };
    let vis = input.vis.clone();
    let ident = &input.ident;
    let mut build_name = input.ident.to_string();
    build_name.push_str("Builder");
    let build_ident = Ident::new(&build_name, Span::call_site());

    let fields = &input.fields;
    let field_idents: Vec<&Ident> = fields.iter().map(|f| f.ident.as_ref().unwrap()).collect();
    let field_names: Vec<String> = field_idents.iter().map(|f| f.to_string()).collect();
    let field_types = fields.iter().map(|f| &f.ty);
    let generic_idents: Vec<Ident> = field_names
        .iter()
        .map(|f| field_name_to_generic(f))
        .collect();

    let all_true_generics = repeat(quote![true]).take(fields.len());
    let all_false_generics = repeat(quote![false]).take(fields.len());

    let setter_fns = fields.iter().enumerate().map(|(active_idx, active_field)| {
        let ident = active_field.ident.as_ref().unwrap();
        let ty = &active_field.ty;

        let fields = field_idents.iter().enumerate().map(|(idx, field)| {
            if active_idx == idx {
                quote! {#field: Some(value)}
            } else {
                quote! {#field: self.#field}
            }
        });

        let generics = generic_idents.iter().enumerate().map(|(idx, generic)| {
            if idx == active_idx {
                quote![true]
            } else {
                quote![#generic]
            }
        });

        quote! {
            pub fn #ident(self, value: #ty) -> #build_ident<#(#generics),*> {
                #build_ident {
                    #(#fields),*
                }
            }
        }
    });

    return quote! {
        #input

        #vis struct #build_ident<#(const #generic_idents: bool = false),*> {
            #(
               #field_idents: Option<#field_types>
            ),*
        }

        impl<#(const #generic_idents: bool),*> std::fmt::Debug for #build_ident<#(#generic_idents),*> {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.debug_struct(#build_name)
                    #(.field(#field_names, &#generic_idents))*
                    .finish()
            }
        }

        impl #build_ident {
            pub fn new() -> #build_ident<#(#all_false_generics),*> {
                return #build_ident {
                    #(#field_idents: None),*
                }
            }
        }

        impl<#(const #generic_idents: bool),*> #build_ident<#(#generic_idents),*> {
            #(#setter_fns)*
        }

        impl #build_ident<#(#all_true_generics),*> {
            pub fn build(self) -> #ident {
                return #ident {
                    #(#field_idents: self.#field_idents.unwrap()),*
                }
            }
        }

    }
    .into();
}

fn field_name_to_generic(field_name: &str) -> Ident {
    let name = format!(
        "T{}",
        field_name
            .split('_')
            .map(|p| {
                if p.len() == 0 {
                    return String::new();
                }
                let mut c = p.chars();
                format!(
                    "{}{}",
                    c.next().unwrap().to_uppercase(),
                    c.collect::<String>()
                )
            })
            .collect::<String>()
    );
    Ident::new(&name, Span::call_site())
}
