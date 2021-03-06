use proc_macro2::{Span, TokenStream};
use syn::{Data, DeriveInput, Field, Fields, Ident, Type};
use utils::{add_extra_ty_param_bound, named_to_vec, unnamed_to_vec};

/// Provides the hook to expand `#[derive(Display)]` into an implementation of `From`
pub fn expand(input: &DeriveInput, trait_name: &str) -> TokenStream {
    let trait_ident = Ident::new(trait_name, Span::call_site());
    let trait_path = &quote!(::std::fmt::#trait_ident);
    let generics = add_extra_ty_param_bound(&input.generics, trait_path);
    let (impl_generics, ty_generics, where_clause) = generics.split_for_impl();
    let input_type = &input.ident;
    let (member, field_type) = match input.data {
        Data::Struct(ref data_struct) => match data_struct.fields {
            Fields::Unnamed(ref fields) => tuple_from_str(trait_name, &unnamed_to_vec(fields)),
            Fields::Named(ref fields) => struct_from_str(trait_name, &named_to_vec(fields)),
            Fields::Unit => panic_one_field(trait_name),
        },
        _ => panic_one_field(trait_name),
    };
    quote!{
        impl#impl_generics #trait_path for #input_type#ty_generics #where_clause
        {
            #[inline]
            fn fmt(&self, formatter: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                <#field_type as #trait_path>::fmt(&#member, formatter)
            }
        }
    }
}

fn panic_one_field(trait_name: &str) -> ! {
    panic!(format!(
        "Only structs with one field can derive({})",
        trait_name
    ))
}

fn tuple_from_str<'a>(trait_name: &str, fields: &[&'a Field]) -> (TokenStream, &'a Type) {
    if fields.len() != 1 {
        panic_one_field(trait_name)
    };
    let field = &fields[0];
    let field_type = &field.ty;
    (quote!(self.0), field_type)
}

fn struct_from_str<'a>(trait_name: &str, fields: &[&'a Field]) -> (TokenStream, &'a Type) {
    if fields.len() != 1 {
        panic_one_field(trait_name)
    };
    let field = &fields[0];
    let field_ident = &field.ident;
    let field_type = &field.ty;
    (quote!(self.#field_ident), field_type)
}
