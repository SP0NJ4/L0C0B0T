use std::ops::Add;

use convert_case::{Case, Casing};
use parsers::CommandFn;
use proc_macro::TokenStream;
use proc_macro2::Span;
use quote::quote;
use syn::{parse_macro_input, Ident};

pub(crate) mod parsers;

/// An attribute macro to define a command.
///
/// The macro takes in a dispatch function and creates a new type that implements
/// the `Command` trait. The dispatch function is called when the command is
/// executed.
///
/// The macro also creates a `Command` instance that can be used to register the
/// command.
///
/// The dispatch function should have the following signature:
///
/// ```ignore
/// fn name(ctx: &mut Context, msg: &Message) -> bool {
///    // ...
/// }
/// ```
///
/// The `name` of the command is the name of the function by default. It can be
/// overridden by passing in a string literal to the macro. The name must be in
/// snake case.
///
/// The instance of the command will be named `NAME` where `NAME` is the name of
/// the command in upper snake case.
#[proc_macro_attribute]
pub fn command(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as CommandFn);

    #[allow(unused)]
    let cmd_attr = input.attributes;

    let fun = input.fun;

    let fun_body = &fun.block;

    let name = if attr.is_empty() {
        fun.sig.ident.to_string()
    } else {
        attr.to_string()
    };

    assert!(name.is_case(Case::Snake), "Command name must be snake case");

    let struct_name = Ident::new(
        name.to_case(Case::Pascal).add("Command").as_str(),
        Span::call_site(),
    );

    let instance_name = Ident::new(
        name.to_case(Case::UpperSnake).add("_COMMAND").as_str(),
        Span::call_site(),
    );

    let output = quote! {
        pub struct #struct_name;

        #[async_trait::async_trait]
        impl Command for #struct_name {
            fn name(&self) -> &'static str {
                #name
            }

            async fn dispatch(&self, ctx: &serenity::prelude::Context, msg: &serenity::model::prelude::Message) -> bool
                #fun_body
        }

        pub const #instance_name: #struct_name = #struct_name;
    };

    output.into()
}
