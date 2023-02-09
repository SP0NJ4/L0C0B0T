use quote::ToTokens;
use syn::{
    parse::{Parse, ParseStream},
    spanned::Spanned,
    Attribute, FnArg, Ident, ItemFn, Token, Type,
};

pub struct CommandFn {
    pub attributes: Vec<Attribute>,
    pub fun: ItemFn,
}

impl Parse for CommandFn {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let attributes = input.call(Attribute::parse_outer)?;
        let fun = input.parse::<ItemFn>()?;

        if !input.is_empty() {
            return Err(syn::Error::new(input.span(), "Commands must be functions"));
        }

        let signature = &fun.sig;

        if signature.asyncness.is_none() {
            return Err(syn::Error::new(
                signature.fn_token.span,
                "Command functions must be async",
            ));
        }

        let input_err: syn::Result<CommandFn> = Err(syn::Error::new(
            signature.inputs.span(),
            format!("Command functions must take two arguments: `ctx` and `msg`"),
        ));

        if signature.inputs.len() != 2 {
            return input_err;
        }

        if let FnArg::Typed(arg) = &signature.inputs[0] {
            if arg.pat.to_token_stream().to_string() != "ctx" {
                return input_err;
            }
        } else {
            return input_err;
        }

        if let FnArg::Typed(arg) = &signature.inputs[1] {
            if arg.pat.to_token_stream().to_string() != "msg" {
                return input_err;
            }
        } else {
            return input_err;
        }

        Ok(CommandFn { attributes, fun })
    }
}

pub struct Setting {
    pub name: String,
    pub ty: Type,
}

impl Parse for Setting {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let name = input.parse::<Ident>()?.to_string();
        input.parse::<Token![:]>()?;
        let type_ = input.parse::<Type>()?;

        Ok(Setting { name, ty: type_ })
    }
}
