use parse::Parser;
use quote::{ToTokens, TokenStreamExt, format_ident, quote};
use syn::{
    braced,
    ext::IdentExt,
    parenthesized,
    parse::{self, Parse, ParseStream},
    punctuated::Punctuated,
    Ident, Token, Type,
};

const PRIMITIVE_NAMES: &'static [&'static str] = &[
    "byte", "i8", "short", "i16", "int", "i32", "long", "i64", "double", "f64", "boolean", "bool",
];
macro_rules! punctuated_to_string {
    ($punctuated: expr, $separator: expr) => {
        $punctuated
            .iter()
            .map(|x| x.into_token_stream().to_string())
            .collect::<Vec<String>>()
            .join($separator)
    };
}

fn type_is_primitive(ty: &Type) -> bool {
    PRIMITIVE_NAMES
        .iter()
        .any(|x| **x == ty.to_token_stream().to_string())
}

#[derive(Debug)]
struct JavaType {
    ty: Type,
    array: bool,
}

impl Parse for JavaType {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let ty: Type = input.parse()?;
        let is_array = syn::group::parse_brackets(input).is_ok();

        Ok(Self {
            ty: ty,
            array: is_array,
        })
    }
}

impl ToTokens for JavaType {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        if self.array {
            let java_type = &self.ty;

            let pass_type = format_ident!("{}Passable", java_type.to_token_stream().to_string());
            // let pass_type = if type_is_primitive(&self.ty) {
            //     self.ty.to_token_stream()
            // } else {
            //     quote!(*mut Value)
            // };

            (quote! {
                JavaArray<#java_type, #pass_type>
            })
            .to_tokens(tokens);
        } else {
            self.ty.to_tokens(tokens);
        };
    }
}

impl JavaType {
    fn to_type(&self) -> Option<Type> {
        syn::parse(self.to_token_stream().into()).ok()
    }
}

#[derive(Debug)]
struct JavaTypedDeclaration {
    ty: JavaType,
    name: Ident,
}

impl Parse for JavaTypedDeclaration {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            ty: input.parse()?,
            name: input.parse()?,
        })
    }
}

impl JavaTypedDeclaration {
    fn to_rust_type_annotation(self) -> proc_macro2::TokenStream {
        let JavaTypedDeclaration { ty, name } = self;
        let tokens = quote! {
            #name: #ty
        };
        tokens
    }
}

#[derive(Debug)]
struct JavaFunctionStub {
    return_type: JavaType,
    rust_name: Ident,
    java_name: Option<Ident>,
    bracket_token: syn::token::Paren,
    args: Punctuated<JavaTypedDeclaration, Token![,]>,
}

impl Parse for JavaFunctionStub {
    fn parse(arg: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(JavaFunctionStub {
            return_type: arg.parse()?,
            rust_name: arg.parse()?,
            java_name: arg.parse().ok(),
            bracket_token: parenthesized!(content in arg),
            args: content.parse_terminated(JavaTypedDeclaration::parse)?,
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
struct JavaQualifiedConstructorStub {
    fully_qualified_type_name: Punctuated<Ident, Token![.]>,
    rust_constructor_name: Ident,
    generics: Option<AngleBracketGenerics>,

    bracket_token: syn::token::Paren,
    args: Punctuated<JavaTypedDeclaration, Token![,]>,
}

impl Parse for JavaQualifiedConstructorStub {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let qualified_parser = Punctuated::<Ident, Token![.]>::parse_separated_nonempty;
        Ok(JavaQualifiedConstructorStub {
            fully_qualified_type_name: qualified_parser(input)?,
            generics: input.parse::<AngleBracketGenerics>().ok(),
            rust_constructor_name: input.parse()?,
            bracket_token: parenthesized!(content in input),
            args: content.parse_terminated(JavaTypedDeclaration::parse)?,
        })
    }
}

#[derive(Debug)]
struct JavaConstructorStub {
    rust_constructor_name: Ident,

    bracket_token: syn::token::Paren,
    args: Punctuated<JavaTypedDeclaration, Token![,]>,
}

impl Parse for JavaConstructorStub {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        Ok(JavaConstructorStub {
            rust_constructor_name: input.parse()?,
            bracket_token: parenthesized!(content in input),
            args: content.parse_terminated(JavaTypedDeclaration::parse)?,
        })
    }
}
#[derive(Debug)]
enum JavaStub {
    //
    JavaConstructorStub(JavaConstructorStub),
    JavaFunctionStub(JavaFunctionStub),
}

impl Parse for JavaStub {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        if input.peek2(syn::token::Paren) {
            input.parse().map(Self::JavaConstructorStub)
        } else {
            input.parse().map(Self::JavaFunctionStub)
        }
    }
}
#[derive(Debug)]
struct Class {
    qualified_name: Punctuated<Ident, Token![.]>,
    generics: Option<AngleBracketGenerics>,

    bracket_token: syn::token::Brace,
    stubs: Punctuated<JavaStub, Token![;]>,
}

impl Parse for Class {
    fn parse(input: ParseStream) -> syn::Result<Self> {
        let content;
        let qualified_parser = Punctuated::<Ident, Token![.]>::parse_separated_nonempty;
        Ok(Self {
            qualified_name: qualified_parser(input)?,
            generics: input.parse::<AngleBracketGenerics>().ok(),
            bracket_token: braced!(content in input),
            stubs: content.parse_terminated(JavaStub::parse)?,
        })
    }
}

fn parse_java_args(
    args: Punctuated<JavaTypedDeclaration, Token![,]>,
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
        Token![->](proc_macro2::Span::call_site()),
        Box::new(return_type),
    )
    .into_token_stream()
}

fn get_return_and_conversion_prefix(return_type: &JavaType) -> proc_macro2::TokenStream {
    let return_type_string = return_type.to_token_stream().to_string();
    let return_type_string = return_type_string.replace("<", "::<"); // When invoking functions, syntax is Type::<GenericArg>::method(), rather than Type<GenericArg>::method().
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

/// new_with_length(int length);
fn quote_constructor_stub(
    fully_qualified_type_name: &Punctuated<Ident, Token![.]>,
    generics: &Option<AngleBracketGenerics>,
    stub: JavaConstructorStub,
) -> proc_macro2::TokenStream {
    let JavaConstructorStub {
        rust_constructor_name,
        args,
        ..
    } = stub;

    let rust_type_name = fully_qualified_type_name
        .last()
        .expect("Could not parse type name.");
    let fully_qualified_type_name = punctuated_to_string!(fully_qualified_type_name, ".");

    let (args, arg_names) = parse_java_args(args);
    let name_lit = syn::LitStr::new(&fully_qualified_type_name, proc_macro2::Span::call_site());

    let generics_and_turbofish = if let Some(generics) = &generics {
        let generics = generics.into_token_stream();
        let res = quote! {
            :: #generics
        };
        Some(res)
    } else {
        None
    };

    quote! {
        pub fn #rust_constructor_name (#(#args),*) -> #rust_type_name #generics {
            let polyglot_type = crate::java_type(#name_lit);
            #rust_type_name #generics_and_turbofish ::from_polyglot_value(crate::new_instance!(polyglot_type #(,#arg_names)*))
        }
    }
}

/// java.util.ArrayList new_with_length(int length);

fn quote_qualified_constructor_stub(
    stub: JavaQualifiedConstructorStub,
) -> proc_macro2::TokenStream {
    let JavaQualifiedConstructorStub {
        fully_qualified_type_name,
        rust_constructor_name,
        args,
        generics,
        ..
    } = stub;
    //
    let rust_type_name = fully_qualified_type_name
        .last()
        .expect("Could not parse type name.");

    let fully_qualified_type_name = punctuated_to_string!(fully_qualified_type_name, ".");

    let (args, arg_names) = parse_java_args(args);
    let name_lit = syn::LitStr::new(&fully_qualified_type_name, proc_macro2::Span::call_site());

    let generics_and_turbofish = if let Some(generics) = &generics {
        let generics = generics.into_token_stream();
        let res = quote! {
            :: #generics
        };
        Some(res)
    } else {
        None //
    };

    quote! {
        pub fn #rust_constructor_name (#(#args),*) -> #rust_type_name #generics {
            let polyglot_type = crate::java_type(#name_lit);
            #rust_type_name #generics_and_turbofish ::from_polyglot_value(crate::new_instance!(polyglot_type #(,#arg_names)*))
        }
    }
}
/**
 `[return_type] name [java_name]([args]);` \
 This function takes a JavaFunctionStub and generates the binding code for it. \
 The following JavaFunctionStub will generate a binding for
 `ArrayList#remove(int index)`, using remove_at as the rust name and `remove` as the java name. (some types shown as strings for clarity):
 ```rust
 JavaFunctionStub {
    return_type: "int",
    rust_name: "remove_at",
    java_name: Some("remove"),
    bracket_token: "()",
    args: ["int index"]
 }
```
 The generated code will look like this:
 ```rust
 pub fn remove_at(&self, index: int) -> E {
    return E::from_polyglot_value({
        unsafe {
            crate::polyglot::internal::polyglot_invoke(
                self.ptr,
                crate::polyglot::internal::make_cstring("remove").as_ptr(),
                crate::polyglot::internal::expect_variadic(index),
            )
        }
    });
}
```
If `java_name` is `None`, it will be assumed to be the same as the provided `rust_name`.
The main purpose of `java_name` is to rename overloaded Java functions, since Rust does not support overloading.
*/
fn quote_function_stub(stub: JavaFunctionStub) -> proc_macro2::TokenStream {
    let JavaFunctionStub {
        return_type,
        rust_name,
        java_name,
        args,
        ..
    } = stub;

    // If no java name was provided, we just assume the java name is the same as the rust function name
    let java_name = java_name
        .map(|x| x.to_string())
        .unwrap_or(rust_name.to_string());

    let (args, arg_names) = parse_java_args(args);

    let conversion_method = get_return_and_conversion_prefix(&return_type);
    let return_token = get_return_token(return_type.to_type().unwrap());

    quote::quote! {
        pub fn #rust_name (&self, #(#args),*) #return_token {
            #conversion_method (crate::invoke_method!(self.ptr, #java_name #(,#arg_names)*)) ;
        }
    }
}

#[proc_macro]
pub fn java_constructor(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parser = Punctuated::<JavaQualifiedConstructorStub, Token![;]>::parse_terminated;
    let stubs = parser.parse(input).expect("Could not parse stubs.");
    let mut output = proc_macro2::TokenStream::new();

    for stub in stubs {
        let constructor_token = quote_qualified_constructor_stub(stub);
        //        println!("{}", constructor_token.to_string());
        constructor_token.to_tokens(&mut output);
    }
    output.into()
}

/**
 `[return_type] rust_name [java_name]([args]);` \
 This function takes a JavaFunctionStub and generates the binding code for it. \
 The following JavaFunctionStub will generate a binding for
 `ArrayList#remove(int index)`, using remove_at as the rust name and `remove` as the java name. (some types shown as strings for clarity):
 ```java
 int remove_at remove(int index);
```
 The generated code will look like this:
 ```rust
 pub fn remove_at(&self, index: int) -> E {
    return E::from_polyglot_value({
        unsafe {
            crate::polyglot::internal::polyglot_invoke(
                self.ptr,
                crate::polyglot::internal::make_cstring("remove").as_ptr(),
                crate::polyglot::internal::expect_variadic(index),
            )
        }
    });
}
```
If `java_name` is `None`, it will be assumed to be the same as the provided `rust_name`.
The main purpose of `java_name` is to rename overloaded Java functions, since Rust does not support overloading.
*/
#[proc_macro]
pub fn java_method(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let parser = Punctuated::<JavaFunctionStub, Token![;]>::parse_terminated;
    let stubs = parser.parse(input).expect("Could not parse stubs");
    let mut output = proc_macro2::TokenStream::new();

    for stub in stubs {
        let function_token = quote_function_stub(stub);
        //        println!("{}", function_token.to_string());
        function_token.to_tokens(&mut output);
    }
    output.into()
}

/// Generates bindings for a Java class, using method and constructor stubs provided in the body.
#[proc_macro]
pub fn class(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    let class = syn::parse_macro_input!(input as Class);
    let mut stubs = vec![];

    let rust_name = class.qualified_name.last().unwrap();

    let mut passable_generics: Vec<Type> = vec![]; // Add new generic types so we can constrain the value of our desired generic types to be Pass + Receive
    let generics = class.generics.map(|generics| {
        for generic_arg in &generics.args {
            passable_generics.push(
                syn::parse_str(&(generic_arg.to_token_stream().to_string() + "Passable")).unwrap(),
            )
        }
        generics
    });

    let generic_bounds = generics.as_ref().map(|AngleBracketGenerics { args, .. }| {
        let mut stream = proc_macro2::TokenStream::new();
        for i in 0..args.len() {
            let ty = &args[i];
            let ty_passable = &passable_generics[i];
            stream.append_all(
                quote! {
                    #ty_passable: Passable,
                    #ty: Pass<#ty_passable> + Receive,
                }
                .to_token_stream(),
            );
        }

        stream
    });

    let generics = generics.map(|mut x| {
        // Combine the passable generics and required ones so we can declare them on the struct
        for i in passable_generics {
            x.args.push(i);
        }
        x
    });

    let mut phantom_field_declarations = vec![];
    let mut phantom_field_initializations = vec![];
    if let Some(ref generics) = generics {
        for type_name in &generics.args {
            let field_name = quote::format_ident!("__phantom_{}", type_name.to_token_stream().to_string());
            phantom_field_declarations.push(quote! {
                #field_name: PhantomData<#type_name>
            });
            phantom_field_initializations.push(quote! {
                #field_name: PhantomData
            })
            
        }
    }

    for stub in class.stubs {
        match stub {
            JavaStub::JavaConstructorStub(stub) => stubs.push(quote_constructor_stub(
                &class.qualified_name,
                &generics,
                stub,
            )),
            JavaStub::JavaFunctionStub(stub) => stubs.push(quote_function_stub(stub)),
        }
    }

    let result = quote! {
        pub struct #rust_name #generics where #generic_bounds
        {
            ptr: *mut Value,
            #(#phantom_field_declarations),*
        }

        impl#generics #rust_name #generics where #generic_bounds {
            #(#stubs)*
        }

        unsafe impl#generics Receive for #rust_name #generics where #generic_bounds
        {
            fn from_polyglot_value(value: *mut Value) -> Self {
                Self {
                    ptr: value,
                    #(#phantom_field_initializations),*
                }
            }
        }

        unsafe impl#generics Pass<*mut Value> for #rust_name #generics where #generic_bounds {
            fn pass(&self) -> *mut Value {
                self.ptr
            }
        }
    };

    result.into()
}
