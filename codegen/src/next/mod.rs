use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{
    parenthesized, parse::Parse, parse2, punctuated::Punctuated, token::Paren, Expr, File, Ident,
    Item, ItemFn, Lit, LitStr, Token,
};

#[derive(Debug, Eq, PartialEq)]
struct Endpoint {
    route: LitStr,
    method: Ident,
    function: Ident,
}

#[derive(Debug, Eq, PartialEq)]
struct Parameter {
    key: Ident,
    equals: Token![=],
    value: Expr,
}

#[derive(Debug, Eq, PartialEq)]
struct Params {
    params: Punctuated<Parameter, Token![,]>,
    paren_token: Paren,
}

impl Parse for Parameter {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        Ok(Self {
            key: input.parse()?,
            equals: input.parse()?,
            value: input.parse()?,
        })
    }
}

impl Parse for Params {
    fn parse(input: syn::parse::ParseStream) -> syn::Result<Self> {
        let content;
        Ok(Self {
            paren_token: parenthesized!(content in input),
            params: content.parse_terminated(Parameter::parse)?,
        })
    }
}

impl Endpoint {
    fn from_item_fn(item: &mut ItemFn) -> Option<Self> {
        let function = item.sig.ident.clone();

        let mut endpoint_index = None;

        // Find the index of an attribute that is an endpoint
        for index in 0..item.attrs.len() {
            // The endpoint ident should be the last segment in the path
            if let Some(segment) = item.attrs[index].path.segments.last() {
                if segment.ident.to_string().as_str() == "endpoint" {
                    // TODO: we should allow multiple endpoint attributes per handler.
                    // We could refactor this to return a Vec<Endpoint> and then check
                    // that the combination of endpoints is valid.
                    if endpoint_index.is_some() {
                        emit_error!(
                            item,
                            "extra endpoint attribute";
                            hint = "There should only be one endpoint annotation per handler function."
                        );
                        return None;
                    }
                    endpoint_index = Some(index);
                }
            } else {
                return None;
            }
        }

        // Strip the endpoint attribute if it exists
        let endpoint = if let Some(index) = endpoint_index {
            item.attrs.remove(index)
        } else {
            // This item does not have an endpoint attribute
            return None;
        };

        // Parse the endpoint's parameters
        let params: Params = match parse2(endpoint.tokens) {
            Ok(params) => params,
            Err(err) => {
                // This will error on invalid parameter syntax
                emit_error!(err.span(), err);
                return None;
            }
        };

        // We'll use the paren span for errors later
        let paren = params.paren_token;

        if params.params.is_empty() {
            emit_error!(
                paren.span,
                "missing endpoint arguments";
                hint = "The endpoint takes two arguments: `endpoint(method = get, route = \"/hello\")`"
            );
            return None;
        }

        // At this point an endpoint with params and valid syntax exists, so we will check for
        // all errors before returning
        let mut has_err = false;

        let mut route = None;
        let mut method = None;

        for Parameter { key, value, .. } in params.params {
            let key_ident = key.clone();
            match key.to_string().as_str() {
                "method" => {
                    if method.is_some() {
                        emit_error!(
                            key_ident,
                            "duplicate endpoint method";
                            hint = "The endpoint `method` should only be set once."
                        );
                        has_err = true;
                    }
                    if let Expr::Path(path) = value {
                        method = Some(path.path.segments[0].ident.clone());
                    };
                }
                "route" => {
                    if route.is_some() {
                        emit_error!(
                            key_ident,
                            "duplicate endpoint route";
                            hint = "The endpoint `route` should only be set once."
                        );
                        has_err = true;
                    }
                    if let Expr::Lit(literal) = value {
                        if let Some(Lit::Str(literal)) = Some(literal.lit) {
                            route = Some(literal);
                        }
                    }
                }
                _ => {
                    emit_error!(
                        key_ident,
                        "invalid endpoint argument";
                        hint = "Only `method` and `route` are valid endpoint arguments."
                    );
                    has_err = true;
                }
            }
        }

        if route.is_none() {
            emit_error!(
                paren.span,
                "no route provided";
                hint = "Add a route to your endpoint: `route = \"/hello\")`"
            );
            has_err = true;
        };

        if method.is_none() {
            emit_error!(
                paren.span,
                "no method provided";
                hint = "Add a method to your endpoint: `method = get`"
            );
            has_err = true;
        };

        if has_err {
            None
        } else {
            // Safe to unwrap because `has_err` is true if `route` or `method` is `None`
            Some(Endpoint {
                route: route.unwrap(),
                method: method.unwrap(),
                function,
            })
        }
    }
}

impl ToTokens for Endpoint {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self {
            route,
            method,
            function,
        } = self;

        match method.to_string().as_str() {
            "get" | "post" | "delete" | "put" | "options" | "head" | "trace" | "patch" => {}
            _ => {
                emit_error!(
                    method,
                    "method is not supported";
                    hint = "Try one of the following: `get`, `post`, `delete`, `put`, `options`, `head`, `trace` or `patch`"
                )
            }
        };

        let route = quote!(.route(#route, axum::routing::#method(#function)));

        route.to_tokens(tokens);
    }
}

#[derive(Debug, Eq, PartialEq)]
pub(crate) struct App {
    endpoints: Vec<Endpoint>,
}

impl App {
    pub(crate) fn from_file(file: &mut File) -> Self {
        let endpoints = file
            .items
            .iter_mut()
            .filter_map(|item| {
                if let Item::Fn(item_fn) = item {
                    Some(item_fn)
                } else {
                    None
                }
            })
            .filter_map(Endpoint::from_item_fn)
            .collect();

        Self { endpoints }
    }
}

impl ToTokens for App {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { endpoints } = self;

        let app = quote!(
            async fn __app(request: http::Request<axum::body::BoxBody>,) -> axum::response::Response
            {
                use tower_service::Service;

                let mut router = axum::Router::new()
                    #(#endpoints)*;

                let response = router.call(request).await.unwrap();

                response
            }
        );

        app.to_tokens(tokens);
    }
}

pub(crate) fn wasi_bindings(app: App) -> proc_macro2::TokenStream {
    quote!(
        #app

        #[no_mangle]
        #[allow(non_snake_case)]
        pub extern "C" fn __SHUTTLE_Axum_call(
            fd_3: std::os::wasi::prelude::RawFd,
            fd_4: std::os::wasi::prelude::RawFd,
            fd_5: std::os::wasi::prelude::RawFd,
        ) {
            use axum::body::HttpBody;
            use std::io::{Read, Write};
            use std::os::wasi::io::FromRawFd;

            println!("inner handler awoken; interacting with fd={},{},{}", fd_3, fd_4, fd_5);

            // file descriptor 3 for reading and writing http parts
            let mut parts_fd = unsafe { std::fs::File::from_raw_fd(fd_3) };

            let reader = std::io::BufReader::new(&mut parts_fd);

            // deserialize request parts from rust messagepack
            let wrapper: shuttle_common::wasm::RequestWrapper = rmp_serde::from_read(reader).unwrap();

            // file descriptor 4 for reading http body into wasm
            let mut body_read_stream = unsafe { std::fs::File::from_raw_fd(fd_4) };

            let mut reader = std::io::BufReader::new(&mut body_read_stream);
            let mut body_buf = Vec::new();
            reader.read_to_end(&mut body_buf).unwrap();

            let body = axum::body::Body::from(body_buf);

            let request = wrapper
                .into_request_builder()
                .body(axum::body::boxed(body))
                .unwrap();

            println!("inner router received request: {:?}", &request);
            let res = futures_executor::block_on(__app(request));

            let (parts, mut body) = res.into_parts();

            // wrap and serialize response parts as rmp
            let response_parts = shuttle_common::wasm::ResponseWrapper::from(parts).into_rmp();

            // write response parts
            parts_fd.write_all(&response_parts).unwrap();

            // file descriptor 5 for writing http body to host
            let mut body_write_stream = unsafe { std::fs::File::from_raw_fd(fd_5) };

            // write body if there is one
            if let Some(body) = futures_executor::block_on(body.data()) {
                body_write_stream.write_all(body.unwrap().as_ref()).unwrap();
            }
        }
    )
}

#[cfg(test)]
mod tests {
    use pretty_assertions::assert_eq;
    use quote::quote;
    use syn::parse_quote;

    use crate::next::{App, Parameter};

    use super::{Endpoint, Params};

    #[test]
    fn endpoint_to_token() {
        let endpoint = Endpoint {
            route: parse_quote!("/hello"),
            method: parse_quote!(get),
            function: parse_quote!(hello),
        };

        let actual = quote!(#endpoint);
        let expected = quote!(.route("/hello", axum::routing::get(hello)));

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn app_to_token() {
        let app = App {
            endpoints: vec![
                Endpoint {
                    route: parse_quote!("/hello"),
                    method: parse_quote!(get),
                    function: parse_quote!(hello),
                },
                Endpoint {
                    route: parse_quote!("/goodbye"),
                    method: parse_quote!(post),
                    function: parse_quote!(goodbye),
                },
            ],
        };

        let actual = quote!(#app);
        let expected = quote!(
            async fn __app(
                request: http::Request<axum::body::BoxBody>,
            ) -> axum::response::Response {
                use tower_service::Service;

                let mut router = axum::Router::new()
                    .route("/hello", axum::routing::get(hello))
                    .route("/goodbye", axum::routing::post(goodbye));

                let response = router.call(request).await.unwrap();

                response
            }
        );

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn parse_endpoint() {
        let cases = vec![
            (
                parse_quote! {
                #[shuttle_codegen::endpoint(method = get, route = "/hello")]
                async fn hello() -> &'static str {
                    "Hello, World!"
                }},
                Some(Endpoint {
                    route: parse_quote!("/hello"),
                    method: parse_quote!(get),
                    function: parse_quote!(hello),
                }),
                0,
            ),
            (
                parse_quote! {
                #[doc = r" This attribute is not an endpoint so keep it"]
                #[shuttle_codegen::endpoint(method = get, route = "/hello")]
                async fn hello() -> &'static str {
                    "Hello, World!"
                }},
                Some(Endpoint {
                    route: parse_quote!("/hello"),
                    method: parse_quote!(get),
                    function: parse_quote!(hello),
                }),
                1,
            ),
            (
                parse_quote! {
                    /// This attribute is not an endpoint so keep it
                    async fn say_hello() -> &'static str {
                        "Hello, World!"
                    }
                },
                None,
                1,
            ),
        ];

        for (mut input, expected, remaining_attributes) in cases {
            let actual = Endpoint::from_item_fn(&mut input);

            assert_eq!(actual, expected);

            // Verify that only endpoint attributes have been stripped
            assert_eq!(input.attrs.len(), remaining_attributes);
        }
    }

    #[test]
    fn parse_parameter() {
        // test method param
        let cases: Vec<(Parameter, Parameter)> = vec![
            (
                // parsing an identifier
                parse_quote! {
                    method = get
                },
                Parameter {
                    key: parse_quote!(method),
                    equals: parse_quote!(=),
                    value: parse_quote!(get),
                },
            ),
            (
                // parsing a string literal
                parse_quote! {
                    route = "/hello"
                },
                Parameter {
                    key: parse_quote!(route),
                    equals: parse_quote!(=),
                    value: parse_quote!("/hello"),
                },
            ),
        ];
        for (actual, expected) in cases {
            assert_eq!(actual, expected);
        }
    }

    #[test]
    fn parse_params() {
        let actual: Params = parse_quote![(method = get, route = "/hello")];

        let mut expected = Params {
            params: Default::default(),
            paren_token: Default::default(),
        };
        expected.params.push(parse_quote!(method = get));
        expected.params.push(parse_quote!(route = "/hello"));

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_app() {
        let mut input = parse_quote! {
            #[shuttle_codegen::endpoint(method = get, route = "/hello")]
            async fn hello() -> &'static str {
                "Hello, World!"
            }

            #[shuttle_codegen::endpoint(method = post, route = "/goodbye")]
            async fn goodbye() -> &'static str {
                "Goodbye, World!"
            }
        };

        let actual = App::from_file(&mut input);
        let expected = App {
            endpoints: vec![
                Endpoint {
                    route: parse_quote!("/hello"),
                    method: parse_quote!(get),
                    function: parse_quote!(hello),
                },
                Endpoint {
                    route: parse_quote!("/goodbye"),
                    method: parse_quote!(post),
                    function: parse_quote!(goodbye),
                },
            ],
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn ui() {
        let t = trybuild::TestCases::new();
        t.compile_fail("tests/ui/next/*.rs");
    }
}
