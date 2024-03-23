use convert_case::{Case, Casing};
use proc_macro::TokenStream;
use syn::{parse_macro_input, parse_quote, DeriveInput, Ident};

#[proc_macro_derive(CDebug)]
pub fn derive_c_debug(item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as DeriveInput);
    let struct_ident = input.ident;

    let fn_name = &format!("{}_debug", struct_ident.to_string().to_case(Case::Snake));
    let fn_ident = syn::Ident::new(fn_name, struct_ident.span());

    quote::quote!(
        #[no_mangle]
        pub extern "C" fn #fn_ident(s: &#struct_ident) {
            println!("{:?}", s);
        }
    )
    .into()
}

#[proc_macro_attribute]
pub fn c_result_fn(_attr: TokenStream, item: TokenStream) -> TokenStream {
    let input = parse_macro_input!(item as syn::ItemFn);
    let function_name = input.sig.ident.clone();

    let mut inner = input.clone();
    let inner_ident = Ident::new(&format!("_inner_{}", function_name), function_name.span());
    inner.sig.ident = inner_ident.clone();

    let mut new_signature = input.sig.clone();
    new_signature.abi = parse_quote!(extern "C");
    new_signature.output = parse_quote!(-> i32);

    let inputs = inner
        .sig
        .inputs
        .iter()
        .map(|input| match input {
            syn::FnArg::Receiver(_) => panic!("Self argument is not supported in ABI"),
            syn::FnArg::Typed(typed) => &typed.pat,
        });

    quote::quote!(
        #inner

        #[no_mangle]
        pub #new_signature {
            match #inner_ident(#(#inputs),*) {
                Ok(_) => 0,
                Err(e) => {
                    eprintln!("{e:?}");
                    1
                }
            }
        }
    )
    .into()
}
