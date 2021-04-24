use parse::Parser;
use quote::{quote, ToTokens};
use syn::{
    parenthesized,
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, Type,
};

#[derive(Debug)]
struct JavaDeclaration {
    ty: Type,
    name: Ident,
}

impl Parse for JavaDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ty: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl JavaDeclaration {
    fn to_rust_type_annotation(self) -> proc_macro2::TokenStream {
        let JavaDeclaration { ty, name } = self;
        let tokens = quote! {
            #name: #ty
        };
        tokens
    }
}

#[derive(Debug)]
struct JavaFunctionStub {
    return_type: Type,
    name: Ident,
    bracket_token: syn::token::Paren,
    args: Punctuated<JavaDeclaration, Token![,]>,
}

impl Parse for JavaFunctionStub {
    fn parse(arg: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(JavaFunctionStub {
            return_type: arg.parse()?,
            name: arg.parse()?,
            bracket_token: parenthesized!(content in arg),
            args: content.parse_terminated(JavaDeclaration::parse)?,
        })
    }
}
#[derive(Debug)]
struct AngleBracketGenerics {
    lbracket: Token![<],
    args: Punctuated<Type, Token![,]>,
    rbracket: Token![>],
}

impl Parse for AngleBracketGenerics {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        Ok(Self {
            lbracket: input.parse()?,
            args: Punctuated::<Type, Token![,]>::parse_separated_nonempty(input)?,
            rbracket: input.parse()?,
        })
    }
}

impl ToTokens for AngleBracketGenerics {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        self.lbracket.to_tokens(tokens);
        self.args.to_tokens(tokens);
        self.rbracket.to_tokens(tokens);
    }
}

#[derive(Debug)]
struct JavaConstructorStub {
    fully_qualified_type_name: Punctuated<Ident, Token![.]>,
    rust_constructor_name: Ident,
    generics: Option<AngleBracketGenerics>,

    bracket_token: syn::token::Paren,
    args: Punctuated<JavaDeclaration, Token![,]>,
}

impl Parse for JavaConstructorStub {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let qualified_parser = Punctuated::<Ident, Token![.]>::parse_separated_nonempty;
        Ok(JavaConstructorStub {
            fully_qualified_type_name: qualified_parser(input)?,
            generics: input.parse::<AngleBracketGenerics>().ok(),
            rust_constructor_name: input.parse()?,
            bracket_token: parenthesized!(content in input),
            args: content.parse_terminated(JavaDeclaration::parse)?,
        })
    }
}

fn parse_java_args(
    args: Punctuated<JavaDeclaration, Token![,]>,
) -> (Vec<proc_macro2::TokenStream>, Vec<proc_macro2::TokenStream>) {
    let arg_names: Vec<proc_macro2::TokenStream> = args
        .iter()
        .map(|decl| decl.name.to_token_stream())
        .collect();

    let args = args
        .into_iter()
        .map(|java_declaration| java_declaration.to_rust_type_annotation())
        .collect::<Vec<proc_macro2::TokenStream>>();

    (args, arg_names)
}

fn get_return_token(return_type: Type) -> proc_macro2::TokenStream {
    let return_type = if return_type.to_token_stream().to_string() == "void" {
        syn::parse_str("()").unwrap()
    } else {
        return_type
    };

    syn::ReturnType::Type(
        Token![->](proc_macro2::Span::mixed_site()),
        Box::new(return_type),
    )
    .into_token_stream()
}

fn get_return_and_conversion_prefix(return_type: &Type) -> proc_macro2::TokenStream {
    let return_type_string = return_type.to_token_stream().to_string();

    if return_type_string == "void" {
        proc_macro2::TokenStream::new()
    } else {
        syn::parse_str(&format!(
            "return {}::from_polyglot_value",
            return_type_string
        ))
        .unwrap()
    }
}

#[proc_macro]
pub fn java_constructor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parser = Punctuated::<JavaConstructorStub, Token![;]>::parse_terminated;
    let stubs = parser.parse(input).unwrap();
    let mut output = proc_macro2::TokenStream::new();

    for stub in stubs {
        let JavaConstructorStub {
            fully_qualified_type_name,
            rust_constructor_name,
            args,
            generics,
            ..
        } = stub;

        let rust_type_name = fully_qualified_type_name.last().unwrap();
        let fully_qualified_type_name = fully_qualified_type_name
            .iter()
            .map(|x| x.into_token_stream().to_string())
            .collect::<Vec<String>>()
            .join(".");

        let (args, arg_names) = parse_java_args(args);
        let name_lit =
            syn::LitStr::new(&fully_qualified_type_name, proc_macro2::Span::mixed_site());

        let generics_and_turbofish = if let Some(generics) = &generics {
            let generics = generics.into_token_stream();
            let res = quote! {
                :: #generics
            };
            Some(res)
        } else {
            None
        }; 

        let constructor_token = quote! {
            pub fn #rust_constructor_name (#(#args),*) -> #rust_type_name #generics {
                let java_type = crate::java_type(#name_lit);
                #rust_type_name #generics_and_turbofish ::from_polyglot_value(crate::new_instance!(java_type #(,#arg_names)*))
            }
        };
        //
//        println!("{}", constructor_token.to_string());
        constructor_token.to_tokens(&mut output);
    }
    output.into()
}

#[proc_macro]
pub fn java_method(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parser = Punctuated::<JavaFunctionStub, Token![;]>::parse_terminated;
    let stubs = parser.parse(input).unwrap();
    let mut output = proc_macro2::TokenStream::new();

    for stub in stubs {
        let JavaFunctionStub {
            return_type,
            name,
            args,
            ..
        } = stub;

        let (args, arg_names) = parse_java_args(args);

        let conversion_method = get_return_and_conversion_prefix(&return_type);
        let return_token = get_return_token(return_type);

        let name_lit = syn::LitStr::new(&name.to_string(), proc_macro2::Span::mixed_site());

        let function_token = quote! {
            pub fn #name (&self, #(#args),*) #return_token {
                #conversion_method (crate::invoke_method!(self.ptr, #name_lit #(,#arg_names)*)) ;
            }
        };

//        println!("{}", function_token.to_string());
        function_token.to_tokens(&mut output);
    }
    output.into()
}
