use proc_macro::TokenStream;
use proc_macro2::Span;
use proc_macro_error2::emit_error;
use quote::{quote, ToTokens};
use syn::{
    parse::Parse, parse_macro_input, parse_quote, punctuated::Punctuated, spanned::Spanned,
    Attribute, Expr, ExprLit, FnArg, Ident, ItemFn, Lit, Pat, PatIdent, Path, ReturnType,
    Signature, Stmt, Token, Type, TypePath,
};

pub(crate) fn tokens(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let mut user_main_fn = parse_macro_input!(item as ItemFn);
    let loader_runner = LoaderAndRunner::from_item_fn(&mut user_main_fn);

    quote! {
        fn main() {
            // manual expansion of #[tokio::main]
            ::shuttle_runtime::tokio::runtime::Builder::new_multi_thread()
                .enable_all()
                .build()
                .unwrap()
                .block_on(async {
                    ::shuttle_runtime::__internals::start(__loader, __runner).await;
                })
        }

        #loader_runner

        #user_main_fn
    }
    .into()
}

struct LoaderAndRunner {
    fn_ident: Ident,
    fn_inputs: Vec<Input>,
    fn_return: TypePath,
}

/// A resource-decorated input to the user's main function
#[derive(Debug, PartialEq)]
struct Input {
    /// The identifier (variable name)
    ident: Ident,
    /// The builder pattern data
    builder: BuilderPattern,
    /// The type declaration
    ty: Type,
}

/// Represents a builder pattern that a resource input gets turned into
#[derive(Debug, PartialEq)]
struct BuilderPattern {
    /// Namespace path to the builder
    path: Path,
    /// Options to call the builder with
    options: BuilderOptions,
}

/// The options (function names, values) to call the builder pattern with
#[derive(Debug, Default, PartialEq)]
struct BuilderOptions {
    options: Punctuated<BuilderOption, Token![,]>,
}

/// One item in the builder pattern
#[derive(Debug, PartialEq)]
struct BuilderOption {
    /// Identifier of the option to set
    ident: Ident,
    /// Value to set option to
    value: Expr,
}

impl Parse for BuilderOptions {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            options: input.parse_terminated(BuilderOption::parse, Token![,])?,
        })
    }
}

impl Parse for BuilderOption {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let ident = input.parse()?;
        let _equal: Token![=] = input.parse()?;
        let value = input.parse()?;

        Ok(Self { ident, value })
    }
}

impl LoaderAndRunner {
    /// Modifies function identifier and inputs while extracting and constructing the Shuttle inputs
    pub(crate) fn from_item_fn(item_fn: &mut ItemFn) -> Option<Self> {
        // prefix the function name to allow any name, such as 'main'
        item_fn.sig.ident = Ident::new(
            &format!("__shuttle_{}", item_fn.sig.ident),
            Span::call_site(),
        );

        let inputs: Vec<_> = item_fn
            .sig
            .inputs
            .iter_mut()
            .filter_map(|input| match input {
                FnArg::Receiver(_) => None,
                FnArg::Typed(typed) => Some(typed),
            })
            .filter_map(|typed| match typed.pat.as_ref() {
                Pat::Ident(ident) => Some((ident, typed.attrs.drain(..).collect(), typed.ty.clone())),
                _ => None,
            })
            .filter_map(|(pat_ident, attrs, ty)| {
                match Self::attribute_to_builder(pat_ident, attrs) {
                    Ok(builder) => Some(Input {
                        ident: pat_ident.ident.clone(),
                        builder,
                        ty: *ty,
                    }),
                    Err(err) => {
                        emit_error!(pat_ident, err; hint = pat_ident.span() => "Try adding an attribute like `#[shuttle_shared_db::Postgres]`");
                        None
                    }
                }
            })
            .collect();

        Self::check_return_type(item_fn.sig.clone()).map(|type_path| Self {
            fn_ident: item_fn.sig.ident.clone(),
            fn_inputs: inputs,
            fn_return: type_path,
        })
    }

    fn check_return_type(signature: Signature) -> Option<TypePath> {
        match signature.output {
            ReturnType::Default => {
                emit_error!(
                    signature,
                    "shuttle_runtime::main functions need to return a service";
                    hint = "See the docs for services with first class support";
                    doc = "https://docs.rs/shuttle-runtime/latest/shuttle_runtime/attr.main.html#shuttle-supported-services"
                );
                None
            }
            ReturnType::Type(_, ty) => match *ty {
                Type::Path(path) => Some(path),
                _ => {
                    emit_error!(
                        ty,
                        "shuttle_runtime::main functions need to return a first class service or 'Result<impl shuttle_service::Service, shuttle_runtime::Error>";
                        hint = "See the docs for services with first class support";
                        doc = "https://docs.rs/shuttle-runtime/latest/shuttle_runtime/attr.main.html#shuttle-supported-services"
                    );
                    None
                }
            },
        }
    }

    fn attribute_to_builder(
        pat_ident: &PatIdent,
        attrs: Vec<Attribute>,
    ) -> syn::Result<BuilderPattern> {
        if attrs.is_empty() {
            return Err(syn::Error::new_spanned(
                pat_ident,
                "resource needs an attribute configuration",
            ));
        }

        let options = if attrs[0].meta.require_list().is_err() {
            Default::default()
        } else {
            attrs[0].parse_args()?
        };

        let builder = BuilderPattern {
            path: attrs[0].path().clone(),
            options,
        };

        Ok(builder)
    }
}

impl ToTokens for LoaderAndRunner {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let fn_ident = &self.fn_ident;
        let return_type = &self.fn_return;
        let inputs_len = self.fn_inputs.len();

        let mut fn_inputs = Vec::with_capacity(inputs_len);
        let mut fn_input_builders = Vec::with_capacity(inputs_len);
        let mut fn_input_builder_options = Vec::with_capacity(inputs_len);
        let mut fn_input_types = Vec::with_capacity(inputs_len);

        // whether any string literals are being used in resource macro args (for secret interpolation)
        let mut needs_vars = false;

        for input in self.fn_inputs.iter() {
            fn_inputs.push(&input.ident);
            fn_input_builders.push(&input.builder.path);
            fn_input_types.push(&input.ty);

            let (methods, values): (Vec<_>, Vec<_>) = input
                .builder
                .options
                .options
                .iter()
                .map(|o| {
                    let value = match &o.value {
                        Expr::Lit(ExprLit {
                            lit: Lit::Str(string),
                            ..
                        }) => {
                            needs_vars = true;
                            quote!(&::shuttle_runtime::__internals::strfmt(#string, &__vars)?)
                        }
                        other => quote!(#other),
                    };

                    (&o.ident, value)
                })
                .unzip();
            let builder_chain = quote!(#(.#methods(#values))*);
            fn_input_builder_options.push(builder_chain);
        }

        // modify output based on if any resource macros are being used
        let (factory_ident, extra_imports): (Ident, Option<Stmt>) = if self.fn_inputs.is_empty() {
            (parse_quote!(_factory), None)
        } else {
            (
                parse_quote!(factory),
                Some(parse_quote!(
                    use ::shuttle_runtime::{ResourceFactory, IntoResource, ResourceInputBuilder};
                )),
            )
        };

        // variables for string interpolating secrets into the attribute macros
        let vars: Option<Stmt> = if needs_vars {
            Some(parse_quote!(
                let __vars = ::std::collections::HashMap::from_iter(
                    factory
                        .get_secrets()
                        .into_iter()
                        .map(|(key, value)| (format!("secrets.{}", key), value.expose().clone()))
                );
            ))
        } else {
            None
        };

        let loader_runner = quote! {
            async fn __loader(
                #factory_ident: ::shuttle_runtime::ResourceFactory,
            ) -> ::std::result::Result<::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>, ::shuttle_runtime::Error> {
                use ::shuttle_runtime::__internals::Context;
                #extra_imports

                #vars

                let mut inputs = Vec::new();
                #(
                    let input: <#fn_input_builders as ResourceInputBuilder>::Input =
                        #fn_input_builders::default()
                        #fn_input_builder_options // `vars` are used here
                        .build(&#factory_ident)
                        .await
                        .context(format!("failed to construct config for {}", stringify!(#fn_input_builders)))?;
                    let json = ::shuttle_runtime::__internals::serde_json::to_vec(&input)
                        .context(format!("failed to serialize config for {}", stringify!(#fn_input_builders)))?;
                    inputs.push(json);
                )*
                Ok(inputs)
            }

            async fn __runner(
                resources: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
            ) -> #return_type {
                use ::shuttle_runtime::__internals::Context;
                #extra_imports

                let mut iter = resources.into_iter();
                #(
                    let x: <#fn_input_builders as ResourceInputBuilder>::Output =
                        ::shuttle_runtime::__internals::serde_json::from_slice(
                            &iter.next().expect("resource list to have correct length")
                        )
                        .context(format!("failed to deserialize output for {}", stringify!(#fn_input_builders)))?;
                    let #fn_inputs: #fn_input_types = x.into_resource()
                        .await
                        .context(format!("failed to initialize {}", stringify!(#fn_input_builders)))?;
                )*

                #fn_ident(#(#fn_inputs),*).await
            }
        };

        loader_runner.to_tokens(tokens);
    }
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;

    use super::*;

    #[test]
    fn from_with_return() {
        let mut input = parse_quote!(
            async fn simple() -> ShuttleAxum {}
        );

        let actual = LoaderAndRunner::from_item_fn(&mut input).unwrap();
        let expected_ident: Ident = parse_quote!(__shuttle_simple);
        let expected_return: TypePath = parse_quote!(ShuttleAxum);

        assert_eq!(actual.fn_ident, expected_ident);
        assert_eq!(actual.fn_inputs, Vec::new());
        assert_eq!(actual.fn_return, expected_return);
    }

    #[test]
    fn from_with_main() {
        let mut input = parse_quote!(
            async fn main() -> ShuttleAxum {}
        );

        let actual = LoaderAndRunner::from_item_fn(&mut input).unwrap();
        let expected_ident: Ident = parse_quote!(__shuttle_main);

        assert_eq!(actual.fn_ident, expected_ident);
    }

    #[test]
    fn parse_fn_inputs() {
        let mut input = parse_quote!(
            async fn complex(#[shuttle_shared_db::Postgres] pool: PgPool) -> ShuttleTide {}
        );

        let actual = LoaderAndRunner::from_item_fn(&mut input).unwrap();
        let expected_ident: Ident = parse_quote!(__shuttle_complex);
        let expected_inputs: Vec<Input> = vec![Input {
            ident: parse_quote!(pool),
            builder: BuilderPattern {
                path: parse_quote!(shuttle_shared_db::Postgres),
                options: Default::default(),
            },
            ty: parse_quote!(PgPool),
        }];

        assert_eq!(actual.fn_ident, expected_ident);
        assert_eq!(actual.fn_inputs, expected_inputs);

        // Make sure attributes was removed from input
        if let syn::FnArg::Typed(param) = input.sig.inputs.first().unwrap() {
            assert!(
                param.attrs.is_empty(),
                "some attributes were not removed: {:?}",
                param.attrs
            );
        } else {
            panic!("expected first input to be typed")
        }
    }

    #[test]
    fn parse_builder_options() {
        let input: BuilderOptions = parse_quote!(
            string = "string_val",
            boolean = true,
            integer = 5,
            float = 2.65,
            enum_variant = SomeEnum::Variant1,
            sensitive = "user:{secrets.password}"
        );

        let mut expected: BuilderOptions = Default::default();
        expected.options.push(parse_quote!(string = "string_val"));
        expected.options.push(parse_quote!(boolean = true));
        expected.options.push(parse_quote!(integer = 5));
        expected.options.push(parse_quote!(float = 2.65));
        expected
            .options
            .push(parse_quote!(enum_variant = SomeEnum::Variant1));
        expected
            .options
            .push(parse_quote!(sensitive = "user:{secrets.password}"));

        assert_eq!(input, expected);
    }

    #[test]
    fn parse_input_with_options() {
        let mut input = parse_quote!(
            async fn complex(
                #[shared::Postgres(size = "10Gb", public = false)] pool: PgPool,
            ) -> ShuttlePoem {
            }
        );

        let actual = LoaderAndRunner::from_item_fn(&mut input).unwrap();
        let expected_ident: Ident = parse_quote!(__shuttle_complex);
        let mut expected_inputs: Vec<Input> = vec![Input {
            ident: parse_quote!(pool),
            builder: BuilderPattern {
                path: parse_quote!(shared::Postgres),
                options: Default::default(),
            },
            ty: parse_quote!(PgPool),
        }];

        expected_inputs[0]
            .builder
            .options
            .options
            .push(parse_quote!(size = "10Gb"));
        expected_inputs[0]
            .builder
            .options
            .options
            .push(parse_quote!(public = false));

        assert_eq!(actual.fn_ident, expected_ident);
        assert_eq!(actual.fn_inputs, expected_inputs);
    }

    #[test]
    fn loader_runner_simple_inputs() {
        let input = LoaderAndRunner {
            fn_ident: parse_quote!(simple),
            fn_inputs: Vec::new(),
            fn_return: parse_quote!(ShuttleSimple),
        };

        let actual = quote!(#input);
        let expected = quote! {
            async fn __loader(
                _factory: ::shuttle_runtime::ResourceFactory,
            ) -> ::std::result::Result<::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>, ::shuttle_runtime::Error> {
                use ::shuttle_runtime::__internals::Context;
                let mut inputs = Vec::new();
                Ok(inputs)
            }

            async fn __runner(
                resources: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
            ) -> ShuttleSimple {
                use ::shuttle_runtime::__internals::Context;
                let mut iter = resources.into_iter();
                simple().await
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn loader_runner_complex_inputs() {
        let input = LoaderAndRunner {
            fn_ident: parse_quote!(__shuttle_complex),
            fn_inputs: vec![
                Input {
                    ident: parse_quote!(pool),
                    builder: BuilderPattern {
                        path: parse_quote!(shuttle_shared_db::Postgres),
                        options: Default::default(),
                    },
                    ty: parse_quote!(sqlx::PgPool),
                },
                Input {
                    ident: parse_quote!(redis),
                    builder: BuilderPattern {
                        path: parse_quote!(shuttle_shared_db::Redis),
                        options: Default::default(),
                    },
                    ty: parse_quote!(something::Redis),
                },
            ],
            fn_return: parse_quote!(ShuttleComplex),
        };

        let actual = quote!(#input);
        let expected = quote! {
            async fn __loader(
                factory: ::shuttle_runtime::ResourceFactory,
            ) -> ::std::result::Result<::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>, ::shuttle_runtime::Error> {
                use ::shuttle_runtime::__internals::Context;
                use ::shuttle_runtime::{ResourceFactory, IntoResource, ResourceInputBuilder};
                let mut inputs = Vec::new();
                let input: <shuttle_shared_db::Postgres as ResourceInputBuilder>::Input =
                    shuttle_shared_db::Postgres::default()
                    .build(&factory)
                    .await
                    .context(format!("failed to construct config for {}", stringify!(shuttle_shared_db::Postgres)))?;
                let json = ::shuttle_runtime::__internals::serde_json::to_vec(&input)
                    .context(format!("failed to serialize config for {}", stringify!(shuttle_shared_db::Postgres)))?;
                inputs.push(json);
                let input: <shuttle_shared_db::Redis as ResourceInputBuilder>::Input =
                    shuttle_shared_db::Redis::default()
                    .build(&factory)
                    .await
                    .context(format!("failed to construct config for {}", stringify!(shuttle_shared_db::Redis)))?;
                let json = ::shuttle_runtime::__internals::serde_json::to_vec(&input)
                    .context(format!("failed to serialize config for {}", stringify!(shuttle_shared_db::Redis)))?;
                inputs.push(json);
                Ok(inputs)
            }

            async fn __runner(
                resources: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
            ) -> ShuttleComplex {

                use ::shuttle_runtime::__internals::Context;
                use ::shuttle_runtime::{ResourceFactory, IntoResource, ResourceInputBuilder};
                let mut iter = resources.into_iter();
                let x: <shuttle_shared_db::Postgres as ResourceInputBuilder>::Output =
                    ::shuttle_runtime::__internals::serde_json::from_slice(
                        &iter.next().expect("resource list to have correct length")
                    )
                    .context(format!("failed to deserialize output for {}", stringify!(shuttle_shared_db::Postgres)))?;
                let pool: sqlx::PgPool = x.into_resource()
                    .await
                    .context(format!("failed to initialize {}", stringify!(shuttle_shared_db::Postgres)))?;
                let x: <shuttle_shared_db::Redis as ResourceInputBuilder>::Output =
                    ::shuttle_runtime::__internals::serde_json::from_slice(
                        &iter.next().expect("resource list to have correct length")
                    )
                    .context(format!("failed to deserialize output for {}", stringify!(shuttle_shared_db::Redis)))?;
                let redis: something::Redis = x.into_resource()
                    .await
                    .context(format!("failed to initialize {}", stringify!(shuttle_shared_db::Redis)))?;

                __shuttle_complex(pool, redis).await
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn output_with_input_options() {
        let mut input = LoaderAndRunner {
            fn_ident: parse_quote!(complex),
            fn_inputs: vec![Input {
                ident: parse_quote!(pool),
                builder: BuilderPattern {
                    path: parse_quote!(shuttle_shared_db::Postgres),
                    options: Default::default(),
                },
                ty: parse_quote!(sqlx::PgPool),
            }],
            fn_return: parse_quote!(ShuttleComplex),
        };

        input.fn_inputs[0]
            .builder
            .options
            .options
            .push(parse_quote!(size = "10Gb"));
        input.fn_inputs[0]
            .builder
            .options
            .options
            .push(parse_quote!(public = false));

        let actual = quote!(#input);
        let expected = quote! {
            async fn __loader(
                factory: ::shuttle_runtime::ResourceFactory,
            ) -> ::std::result::Result<::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>, ::shuttle_runtime::Error> {
                use ::shuttle_runtime::__internals::Context;
                use ::shuttle_runtime::{ResourceFactory, IntoResource, ResourceInputBuilder};
                let __vars = ::std::collections::HashMap::from_iter(
                    factory
                        .get_secrets()
                        .into_iter()
                        .map(|(key, value)| (format!("secrets.{}", key), value.expose().clone()))
                );
                let mut inputs = Vec::new();
                let input: <shuttle_shared_db::Postgres as ResourceInputBuilder>::Input =
                    shuttle_shared_db::Postgres::default()
                    .size(&::shuttle_runtime::__internals::strfmt("10Gb", &__vars)?).public(false)
                    .build(&factory)
                    .await
                    .context(format!("failed to construct config for {}", stringify!(shuttle_shared_db::Postgres)))?;
                let json = ::shuttle_runtime::__internals::serde_json::to_vec(&input)
                    .context(format!("failed to serialize config for {}", stringify!(shuttle_shared_db::Postgres)))?;
                inputs.push(json);
                Ok(inputs)
            }
            async fn __runner(
                resources: ::std::vec::Vec<::std::vec::Vec<::core::primitive::u8>>,
            ) -> ShuttleComplex {
                use ::shuttle_runtime::__internals::Context;
                use ::shuttle_runtime::{ResourceFactory, IntoResource, ResourceInputBuilder};
                let mut iter = resources.into_iter();
                let x: <shuttle_shared_db::Postgres as ResourceInputBuilder>::Output =
                    ::shuttle_runtime::__internals::serde_json::from_slice(
                        &iter.next().expect("resource list to have correct length")
                    )
                    .context(format!("failed to deserialize output for {}", stringify!(shuttle_shared_db::Postgres)))?;
                let pool: sqlx::PgPool = x.into_resource()
                    .await
                    .context(format!("failed to initialize {}", stringify!(shuttle_shared_db::Postgres)))?;

                complex(pool).await
            }
        };

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn compiler_output() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/compiler_output/*.rs");
    }
}
