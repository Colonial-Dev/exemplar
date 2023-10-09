mod codegen;
mod util;

use proc_macro::TokenStream;
use proc_macro_error::*;

use proc_macro2::Ident;
use proc_macro2::Literal;
use proc_macro2::TokenStream as QuoteStream;

use quote::*;

use syn::*;
use syn::spanned::Spanned;

use crate::util::Derivee;

#[proc_macro_error]
#[proc_macro_derive(
    Model,
    attributes(table, check, bind, extr, column)
)]
pub fn derive_model(input: TokenStream) -> TokenStream {
    let ast = parse_macro_input!(input as DeriveInput);

    let name = &ast.ident;

    let Data::Struct(data) = &ast.data else {
        abort_call_site!(
            "Model can only be derived for struct types.";
            note = "Enums and unions are not supported...";
            hint = "...but they can be embedded in a Model struct if they implement ToSql.";
        )
    };

    let Fields::Named(fields) = &data.fields else {
        abort_call_site!(
            "Model can only be derived for structs with named fields.";
            note = "Tuple and unit structs are not supported.";
        )
    };

    let fields: Vec<_> = fields
        .named
        .iter()
        .collect();

    let table = util::get_table_name(&ast);
    let schema = util::get_check_path(&ast);

    let derivee = Derivee {
        name: name.to_owned(),
        table,
        fields,
        schema
    };

    let from_row            = codegen::from_row(&derivee);
    let (insert, insert_or) = codegen::inserts(&derivee);
    let as_params           = codegen::as_params(&derivee);
    let check_test          = codegen::check_test(&derivee);
    
    quote! {
        #[automatically_derived]
        impl ::exemplar::Model for #name {
            #from_row
            #insert
            #insert_or
            #as_params
        }

        #[automatically_derived]
        impl<'a> ::std::convert::TryFrom<&'a ::rusqlite::Row<'_>> for #name {
            type Error = ::rusqlite::Error;

            fn try_from(value: &'a ::rusqlite::Row) -> Result<Self, Self::Error> {
                Self::from_row(value)
            }
        }

        #check_test
    }
    .into()
}

