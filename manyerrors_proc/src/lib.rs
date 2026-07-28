use proc_macro::TokenStream;
use quote::quote;
use syn::{Expr, Ident, ItemFn, ReturnType, Token, parse_macro_input, punctuated::Punctuated};

///Take an iterator, stash all errors in the iterator, and return an iterator over all the
///`Ok`s.
/// 
///Works as well with anything that implements IntoIterator.
/// 
///If you need to return early if any items are `Err()` while still collecting all errors,
///use `.lazy_errs().collect()?` instead.
/// 
/// # example
///
///```
/// assert_eq!(
///     iter_stash!(vec![Ok(1),Ok(2),Err(String::from("error")),Ok(3)]).collect::<Vec<_>>(),
///     vec![1,2,3]
/// );
///```
#[proc_macro]
pub fn iter_stash(input: TokenStream) -> TokenStream {
    let p: proc_macro2::TokenStream = input.into();
    quote! {
        manyerrors::IterStash::new(#p,__manyerrors_internal_error_stash__)
    }.into()
}

///Stash an error or a result with an error to be returned later.
///
///Running `stash!()` on an `Ok()` value will do nothing and return `Some()`, while
///running `stash!()` on an `Err()` value will stash the error and return `None`.
/// 
///With more than one argument, `stash!` will combine the result of `stash!`ing 
///all of the arguments such that `stash!(a,b,c)` return `Some((a2,b2,c2))` if `a`, 
///`b`, and `c` are all `Ok`.
#[proc_macro]
pub fn stash(input: TokenStream) -> TokenStream {
    let z = parse_macro_input!(input with Punctuated<Expr,Token![,]>::parse_terminated);
    let mut r: Option<proc_macro2::TokenStream> = None;
    for i in z {
        let v = quote!{{
            use manyerrors::StashErrors;
            #i.stash_errs(__manyerrors_internal_error_stash__)
        }};
        if let Some(k) = r {
            r = Some(quote!(#k.zip(#v)));
        } else {
            r = Some(v);
        }
    }
    r.unwrap().into()
}

///Takes the input, runs it like typical rust code, but the ? operator only short circuits the inner body
/// of the macro instead of the entire function. 
/// 
/// If the input returns a value implicitly (via an expression that doesn't
/// end in a `;`), this value is wrapped in a `Some()` value and returned from the macro.
/// 
/// # example
/// 
/// ```
/// for i in some_list {
///     stash_fn! {
///         let new_value = some_operation(i)?;
///         println("New value: {new_value}");
///     }
/// }
/// ```
#[proc_macro]
pub fn stash_fn(input: TokenStream) -> TokenStream {
    let i: proc_macro2::TokenStream = input.into();
    quote! {
        {
            let __manyerrors_internal_closure__ = || -> Result<_,manyerrors::Errors<_>> {
                let __manyerrors_internal_ok_val__ = { #i };
                Ok( __manyerrors_internal_ok_val__ )
            };
            manyerrors::stash!(
                __manyerrors_internal_closure__()
            )
        }
    }.into()
}

///Main attribute macro for functions that return `Result<O,Errors<T>>`.
///Allows use of the `stash!` macro for stashing errors to be returned at the end of a function.
/// 
///After the function returns, if any errors are stashed they will be appended to the
///returned error value or replace the returned `Ok()` value.
///
///The value T inside `#[manyerrors(T)]` refers to the type returned to in the `Err()`
///case, ie a function marked with `#[manyerrors(T)]` returns `Result<O,Errors<T>>`.
///If no value is set, this value is assumed to be `anyhow::Error` (requires the
///crate feature `anyhow`).
///
/// # example
///
///With an error type of `String`
///
///```
///#[manyerrors(String)]
///fn div(a: i32, b: i32) -> Result<i32,Errors<String>> {
///    if b == 0 {
///        return Err(err(String::from("Can't divide by zero!")));
///    }
///    Ok(a / b)
///} 
///```
/// 
///Using anyhow
/// 
///```
///use manyerrors::anyhow::Result;
///#[manyerrors]
///fn div(a: i32, b: i32) -> Result<i32> {
///    if b == 0 {
///        return Err(err(anyhow!("Can't divide by zero!")));
///    }
///    Ok(a / b)
///} 
///```
#[proc_macro_attribute]
pub fn manyerrors(a: TokenStream, input: TokenStream) -> TokenStream {
    let f = parse_macro_input!(input as ItemFn);
    let attrs = f.attrs;
    let sig = f.sig;
    let body = f.block;
    let _mods = f.modifiers;
    let vis = f.vis;

    let etype: proc_macro2::TokenStream = if a.is_empty() { quote! {anyhow::Error} } else { a.into() };

    let inps = sig.inputs.iter();
    let mut helper_namespace = quote!{};
    let inps2 = inps.clone().map(|z| match z {
        syn::FnArg::Receiver(_a) => {
            helper_namespace = quote! {Self::};
            quote! { self }
        },
        syn::FnArg::Typed(a) => { let z = a.pat.clone(); quote! {#z} }
    }).collect::<Vec<_>>();

    //panic!("{:#?}",quote!(#(#inps2),* &mut __manyerrors_internal_error_stash__));

    let out = sig.output.clone();

    let ReturnType::Type(_arrow,_t) = &out else {
        return quote! { compile_error!("functions annotated with #[manyerrors] must have a return type of Result!"); }.into()
    };

    let helper_name = Ident::new(
        &format!("__manyerrors_internal_helper_{}__",sig.ident.to_string()),
        proc_macro2::Span::call_site()
    );

    let comma = if inps2.is_empty() { quote!{} } else { quote!{,} };

    let helper_fn = quote! {
        fn #helper_name( #(#inps),* #comma __manyerrors_internal_error_stash__: &mut manyerrors::Errors<#etype> ) #out #body
    };

    let (helper_fn_b, helper_fn_a) = if helper_namespace.is_empty() {
        (helper_fn, quote!{})
    } else {
        (quote!{}, helper_fn)
    };

    quote! {
        #helper_fn_a
        #(#attrs)* #vis #sig {
            #helper_fn_b
            let mut __manyerrors_internal_error_stash__ = manyerrors::Errors::new();
            let __helper_result = #helper_namespace #helper_name(#(#inps2),* #comma &mut __manyerrors_internal_error_stash__);
            __manyerrors_internal_error_stash__.add_result(__helper_result)
        }
    }.into()
}