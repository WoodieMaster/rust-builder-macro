use std::iter::repeat_n;

use proc_macro::TokenStream as Pm1TokenStream;
use proc_macro2::{Span, TokenStream as Pm2TokenStream};

use quote::{ToTokens, quote};
use syn::{
    ConstParam, Expr, Ident, Lifetime, LifetimeParam, Token, TypeParam,
    parse::{Parse, ParseBuffer, ParseStream},
    spanned::Spanned,
};

#[derive(Clone, Debug, PartialEq, Default)]
enum AttrDebugOption {
    #[default]
    Simple,
    Full,
}

impl Parse for AttrDebugOption {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident: syn::Ident = input
            .parse()
            .map_err(|e| syn::Error::new(e.span(), "Requires single identifier"))?;

        if ident == "simple" {
            Ok(Self::Simple)
        } else if ident == "full" {
            Ok(Self::Full)
        } else {
            Err(syn::Error::new(ident.span(), "Expected valid Debug Option"))
        }
    }
}

#[derive(Clone, Debug, Default)]
struct Attributes {
    debug: AttrDebugOption,
    use_default: bool,
    builder_fn: bool,
}

impl Parse for Attributes {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let mut attr = Self::default();

        while !input.is_empty() {
            let ident: Ident = input
                .parse()
                .map_err(|e| syn::Error::new(e.span(), "Requires single identifier"))?;

            let mut value: Option<Expr> = None;

            if input.peek(Token![=]) {
                input.parse::<Token![=]>().expect("Token `=` peeked");
                value = Some(
                    input
                        .parse()
                        .map_err(|e| syn::Error::new(e.span(), "Value must be an expression"))?,
                );
            }

            if ident == "debug" {
                attr.debug = syn::parse::Parser::parse2(
                    |input: ParseStream<'_>| AttrDebugOption::parse(input),
                    value
                        .ok_or_else(|| {
                            syn::Error::new(input.span(), "Debug attribute is not a flag")
                        })?
                        .into_token_stream(),
                )?;
            } else if ident == "use_default" {
                if let Some(expr) = value {
                    attr.use_default = syn::parse2::<syn::LitBool>(expr.into_token_stream())
                        .map_err(|e| {
                            syn::Error::new(
                                e.span(),
                                "use_default requires a boolean for its value",
                            )
                        })?
                        .value;
                } else {
                    attr.use_default = true;
                }
            } else if ident == "builder_fn" {
                if let Some(expr) = value {
                    attr.builder_fn = syn::parse2::<syn::LitBool>(expr.into_token_stream())
                        .map_err(|e| {
                            syn::Error::new(
                                e.span(),
                                "use_default requires a boolean for its value",
                            )
                        })?
                        .value;
                } else {
                    attr.builder_fn = true;
                }
            }

            if input.peek(Token![,]) {
                input.parse::<Token![,]>().expect("Token `,` peeked");
            } else if !input.is_empty() {
                return Err(syn::Error::new(input.span(), "Expected `,`"));
            }
        }

        Ok(attr)
    }
}

#[proc_macro_attribute]
pub fn builder(attr: Pm1TokenStream, item: Pm1TokenStream) -> Pm1TokenStream {
    builder_impl(attr.into(), item.into()).into()
}

fn builder_impl(attr: Pm2TokenStream, input: Pm2TokenStream) -> Pm2TokenStream {
    let input: syn::ItemStruct = match syn::parse2(input) {
        Ok(is) => is,
        Err(e) => {
            return e.to_compile_error();
        }
    };

    let attr_result = syn::parse::Parser::parse2(|b: &ParseBuffer<'_>| Attributes::parse(b), attr);
    let args = match attr_result {
        Ok(v) => v,
        Err(e) => {
            return e.to_compile_error();
        }
    };

    let vis = &input.vis;
    let ident = &input.ident;
    let lifetimes: Vec<&LifetimeParam> = input.generics.lifetimes().collect();
    let lifetime_idents: Vec<&Lifetime> = lifetimes.iter().map(|l| &l.lifetime).collect();
    let const_params: Vec<&ConstParam> = input.generics.const_params().collect();
    let const_param_idents: Vec<&Ident> = const_params.iter().map(|l| &l.ident).collect();
    let type_params: Vec<&TypeParam> = input.generics.type_params().collect();
    let type_param_idents: Vec<&Ident> = type_params.iter().map(|l| &l.ident).collect();

    let mut build_name = input.ident.to_string();
    build_name.push_str("Builder");
    let build_ident = Ident::new(&build_name, Span::call_site());

    let fields = &input.fields;

    let field_idents: Vec<&Ident> = {
        let optional =
            fields.iter().map(|f| f.ident.as_ref()).try_fold(
                Vec::<&Ident>::with_capacity(fields.len()),
                |mut acc, el| match el {
                    Some(i) => {
                        acc.push(i);
                        Ok(acc)
                    }
                    None => Err(syn::Error::new(input.span(), "Tuples are not supported")
                        .into_compile_error()),
                },
            );

        match optional {
            Ok(v) => v,
            Err(e) => return e,
        }
    };
    let field_names: Vec<String> = field_idents.iter().map(|f| f.to_string()).collect();
    let field_types = fields.iter().map(|f| &f.ty);
    let generic_idents: Vec<Ident> = field_names
        .iter()
        .map(|f| field_name_to_generic(f))
        .collect();

    let all_true_generics = repeat_n(quote![true], fields.len());
    let all_false_generics = repeat_n(quote![false], fields.len());

    let generic_definition = quote!(#(#lifetimes,)* #(#const_params,)* #(#type_params,)*);
    let generic_use =
        quote!(#(#lifetime_idents,)* #(#const_param_idents,)* #(#type_param_idents,)*);

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
            pub fn #ident(self, value: #ty) -> #build_ident<#generic_use #(#generics),*> {
                #build_ident {
                    #(#fields),*
                }
            }
        }
    });

    let debug = match args.debug {
        AttrDebugOption::Full => quote! {
            impl<#generic_definition #(const #generic_idents: bool),*> std::fmt::Debug for #build_ident<#generic_use #(#generic_idents),*> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(#build_name)
                        #(.field(#field_names, &self.#field_idents))*
                        .finish()
                }
            }
        },
        AttrDebugOption::Simple => quote! {
            impl<#generic_definition #(const #generic_idents: bool),*> std::fmt::Debug for #build_ident<#generic_use #(#generic_idents),*> {
                fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                    f.debug_struct(#build_name)
                        #(.field(#field_names, &#generic_idents))*
                        .finish()
                }
            }
        },
    };

    let build_with_default = if args.use_default {
        quote! {
            fn build_with_default(self) -> #ident<#generic_use> {
                let default: #ident<#generic_use> = core::default::Default::default();

                #ident {
                    #(#field_idents: self.#field_idents.unwrap_or(default.#field_idents)),*
                }
            }
        }
    } else {
        quote!()
    };

    let builder_fn = if args.builder_fn {
        quote! {
            impl<#generic_definition> #ident<#generic_use> {
                fn builder() -> #build_ident<#generic_use> {#build_ident::new()}
            }
        }
    } else {
        quote!()
    };

    let output = quote! {
        #input

        #vis struct #build_ident<#generic_definition #(const #generic_idents: bool = false),*> {
            #(
               #field_idents: Option<#field_types>
            ),*
        }

        #builder_fn

        #debug

        impl<#generic_definition> #build_ident<#generic_use> {
            pub fn new() -> #build_ident<#generic_use #(#all_false_generics),*> {
                return #build_ident {
                    #(#field_idents: None),*
                }
            }
        }

        impl<#generic_definition #(const #generic_idents: bool),*> #build_ident<#generic_use #(#generic_idents),*> {
            #(#setter_fns)*

            #build_with_default
        }

        impl<#generic_definition> #build_ident<#generic_use #(#all_true_generics),*> {
            pub fn build(self) -> #ident<#generic_use> {
                return #ident {
                    #(#field_idents: self.#field_idents.unwrap()),*
                }
            }
        }

    };

    output
}

fn field_name_to_generic(field_name: &str) -> Ident {
    let name = format!(
        "C_{}",
        field_name
            .split('_')
            .map(|p| {
                if p.is_empty() {
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
