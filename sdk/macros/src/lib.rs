use proc_macro::TokenStream;
use proc_macro2::{Ident, Span};
use quote::{quote, ToTokens};

use syn::parse::Parser;

use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};

// syn::AttributeArgs does not implement syn::Parse
type AttributeArgs = syn::punctuated::Punctuated<syn::Meta, syn::Token![,]>;

fn parse_methods(args: TokenStream) -> syn::Result<Vec<String>> {
    let mut idents = vec![];
    if let Ok(args) = AttributeArgs::parse_terminated.parse2(args.into()) {
        for arg in args {
            let ident = match arg {
                syn::Meta::NameValue(namevalue) => namevalue
                    .path
                    .get_ident()
                    .ok_or_else(|| {
                        syn::Error::new_spanned(&namevalue, "Must have specified ident")
                    })?
                    .to_string()
                    .to_uppercase(),
                syn::Meta::Path(path) => path
                    .get_ident()
                    .ok_or_else(|| syn::Error::new_spanned(&path, "Must have specified ident"))?
                    .to_string()
                    .to_uppercase(),
                other => {
                    return Err(syn::Error::new_spanned(
                        other,
                        "Unknown attribute inside the macro",
                    ))
                }
            };
            match ident.as_str() {
                "GET" | "HEAD" | "POST" | "PUT" | "DELETE" | "OPTIONS" | "TRACE" | "PATCH" => {}
                name => {
                    let msg = format!(
                            "Unknown method attribute {} is specified; expected one of: `GET`, `HEAD`, `POST`, `PUT`, `DELETE`, `OPTIONS`, `TRACE`, `PATCH`",
                            name,
                        );
                    return Err(syn::Error::new_spanned(name, msg));
                }
            }
            idents.push(ident);
        }
    }
    Ok(idents)
}

#[proc_macro_attribute]
pub fn request_handler(args: TokenStream, item: TokenStream) -> TokenStream {
    let ast: syn::ItemFn = syn::parse(item).unwrap();
    let func_ident = ast.sig.ident.clone();

    let gen = match ast.sig.inputs.len() {
        0 => {
            quote! {
                #[no_mangle]
                #[tokio::main(flavor = "current_thread")]
                pub async fn __webhook__on_request_received() {
                    #func_ident().await;
                }
            }
        }
        4 => {
            let extern_mod_name = format!("webhook_flows_macros_{}", rand_ident());
            let extern_mod_ident = Ident::new(&extern_mod_name, Span::call_site());
            let request_fn_name = format!("__request_{}", rand_ident());
            let request_fn_ident = Ident::new(&request_fn_name, Span::call_site());

            let gen = quote! {
                mod #extern_mod_ident {
                    extern "C" {
                        pub fn get_event_headers_length() -> i32;
                        pub fn get_event_headers(p: *mut u8) -> i32;
                        pub fn get_event_query_length() -> i32;
                        pub fn get_event_query(p: *mut u8) -> i32;
                        pub fn get_event_subpath_length() -> i32;
                        pub fn get_event_subpath(p: *mut u8) -> i32;
                        pub fn get_event_body_length() -> i32;
                        pub fn get_event_body(p: *mut u8) -> i32;
                    }
                }

                fn #request_fn_ident() -> Option<(Vec<(String, String)>, String, HashMap<String, Value>, Vec<u8>)> {
                    unsafe {
                        let l = #extern_mod_ident::get_event_headers_length();
                        let mut event_headers = Vec::<u8>::with_capacity(l as usize);
                        let c = #extern_mod_ident::get_event_headers(event_headers.as_mut_ptr());
                        assert!(c == l);
                        event_headers.set_len(c as usize);
                        let event_headers = serde_json::from_slice(&event_headers).unwrap();

                        let l = #extern_mod_ident::get_event_query_length();
                        let mut event_query = Vec::<u8>::with_capacity(l as usize);
                        let c = #extern_mod_ident::get_event_query(event_query.as_mut_ptr());
                        assert!(c == l);
                        event_query.set_len(c as usize);
                        let event_query = serde_json::from_slice(&event_query).unwrap();

                        let l = #extern_mod_ident::get_event_subpath_length();
                        let mut event_subpath = Vec::<u8>::with_capacity(l as usize);
                        let c = #extern_mod_ident::get_event_subpath(event_subpath.as_mut_ptr());
                        assert!(c == l);
                        event_subpath.set_len(c as usize);
                        let event_subpath = String::from_utf8_lossy(&event_subpath).into_owned();

                        let l = #extern_mod_ident::get_event_body_length();
                        let mut event_body = Vec::<u8>::with_capacity(l as usize);
                        let c = #extern_mod_ident::get_event_body(event_body.as_mut_ptr());
                        assert!(c == l);
                        event_body.set_len(c as usize);

                        Some((event_headers, event_subpath, event_query, event_body))
                    }
                }
            };

            let methods = parse_methods(args).unwrap();

            match methods.len() > 0 {
                true => {
                    let mut q = quote! {};
                    for m in methods.iter() {
                        let fn_name = format!("__webhook__on_request_received_{}", m);
                        let fn_ident = Ident::new(&fn_name, Span::call_site());
                        q = quote! {
                            #q
                            #[no_mangle]
                            #[tokio::main(flavor = "current_thread")]
                            pub async fn #fn_ident() {
                                if let Some((headers, subpath, qry, body)) = #request_fn_ident() {
                                    #func_ident(headers, subpath, qry, body).await;
                                }
                            }
                        };
                    }
                    quote! {
                        #gen
                        #q
                    }
                }
                false => {
                    quote! {
                        #gen

                        #[no_mangle]
                        #[tokio::main(flavor = "current_thread")]
                        pub async fn __webhook__on_request_received() {
                            if let Some((headers, subpath, qry, body)) = #request_fn_ident() {
                                #func_ident(headers, subpath, qry, body).await;
                            }
                        }
                    }
                }
            }
        }
        _ => {
            panic!("Not compatible fn");
        }
    };

    let ori_run_str = ast.to_token_stream().to_string();
    let x = gen.to_string() + &ori_run_str;
    x.parse().unwrap()
}

fn rand_ident() -> String {
    thread_rng()
        .sample_iter(&Alphanumeric)
        .take(3)
        .map(char::from)
        .collect::<String>()
        .to_lowercase()
}
