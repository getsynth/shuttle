use proc_macro_error::emit_error;
use quote::{quote, ToTokens};
use syn::{
    braced, bracketed, parenthesized, parse::Parse, parse2, parse_macro_input, parse_quote,
    punctuated::Punctuated, token::Paren, Expr, Ident, ItemFn, Lit, LitStr, Token,
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
    fn from_item_fn(item: ItemFn) -> syn::Result<Self> {
        let function = item.sig.ident;

        let params = item.attrs[0].tokens.clone();
        let params: Params = parse2(params)?;

        let mut route = None;
        let mut method = None;

        for Parameter { key, value, .. } in params.params {
            match key.to_string().as_str() {
                "method" => {
                    if let Expr::Path(path) = value {
                        method = Some(path.path.segments[0].ident.clone());
                    };
                }
                "route" => {
                    if let Expr::Lit(literal) = value {
                        if let Some(Lit::Str(literal)) = Some(literal.lit) {
                            route = Some(literal);
                        }
                    }
                }
                _ => todo!(),
            }
        }

        let route = if let Some(route) = route {
            route
        } else {
            todo!()
        };

        let method = if let Some(method) = method {
            method
        } else {
            todo!()
        };

        Ok(Endpoint {
            route,
            method,
            function,
        })
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

pub(crate) struct App {
    endpoints: Vec<Endpoint>,
}

impl ToTokens for App {
    fn to_tokens(&self, tokens: &mut proc_macro2::TokenStream) {
        let Self { endpoints } = self;

        let app = quote!(
            async fn __app<B>(request: http::Request<B>) -> axum::response::Response
            where
                B: axum::body::HttpBody + Send + 'static,
            {
                use tower_service::Service;

                let mut router = axum::Router::new()
                    #(#endpoints)*
                    .into_service();

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
        ) {
            use axum::body::HttpBody;
            use std::io::{Read, Write};
            use std::os::wasi::io::FromRawFd;

            println!("inner handler awoken; interacting with fd={fd_3},{fd_4}");

            // file descriptor 3 for reading and writing http parts
            let mut parts_fd = unsafe { std::fs::File::from_raw_fd(fd_3) };

            let reader = std::io::BufReader::new(&mut parts_fd);

            // deserialize request parts from rust messagepack
            let wrapper: shuttle_common::wasm::RequestWrapper = rmp_serde::from_read(reader).unwrap();

            // file descriptor 4 for reading and writing http body
            let mut body_fd = unsafe { std::fs::File::from_raw_fd(fd_4) };

            // read body from host
            let mut body_buf = Vec::new();
            let mut c_buf: [u8; 1] = [0; 1];
            loop {
                body_fd.read(&mut c_buf).unwrap();
                if c_buf[0] == 0 {
                    break;
                } else {
                    body_buf.push(c_buf[0]);
                }
            }

            let request: http::Request<axum::body::Body> = wrapper
                .into_request_builder()
                .body(body_buf.into())
                .unwrap();

            println!("inner router received request: {:?}", &request);
            let res = futures_executor::block_on(__app(request));

            let (parts, mut body) = res.into_parts();

            // wrap and serialize response parts as rmp
            let response_parts = shuttle_common::wasm::ResponseWrapper::from(parts).into_rmp();

            // write response parts
            parts_fd.write_all(&response_parts).unwrap();

            // write body if there is one
            if let Some(body) = futures_executor::block_on(body.data()) {
                body_fd.write_all(body.unwrap().as_ref()).unwrap();
            }
            // signal to the reader that end of file has been reached
            body_fd.write(&[0]).unwrap();
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
            async fn __app<B>(request: http::Request<B>) -> axum::response::Response
            where
                B: axum::body::HttpBody + Send + 'static,
            {
                use tower_service::Service;

                let mut router = axum::Router::new()
                    .route("/hello", axum::routing::get(hello))
                    .route("/goodbye", axum::routing::post(goodbye))
                    .into_service();

                let response = router.call(request).await.unwrap();

                response
            }
        );

        assert_eq!(actual.to_string(), expected.to_string());
    }

    #[test]
    fn parse_endpoint() {
        let input = parse_quote! {
            #[shuttle_codegen::endpoint(method = get, route = "/hello")]
            async fn hello() -> &'static str {
                "Hello, World!"
            }
        };

        let actual = Endpoint::from_item_fn(input).unwrap();
        let expected = Endpoint {
            route: parse_quote!("/hello"),
            method: parse_quote!(get),
            function: parse_quote!(hello),
        };

        assert_eq!(actual, expected);
    }

    #[test]
    fn parse_parameter() {
        // test method param
        let tests: Vec<(Parameter, Parameter)> = vec![
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
        for (actual, expected) in tests {
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
}
