use proc_macro::TokenStream as TS;
use proc_macro2::Span;
use quote::quote;
use syn::{Item, LitStr, parse_macro_input};

#[proc_macro_attribute]
pub fn link(attr: TS, item: TS) -> TS {
    let attr = parse_macro_input!(attr as LitStr);
    let item = parse_macro_input!(item as Item);

    let val = &attr.value();

    if !val.is_ascii() {
        panic!("value must be an ASCII string");
    }

    let linux_lit = LitStr::new(&linux::section(val), Span::call_site());
    let windows_lit = LitStr::new(&windows::section(val), Span::call_site());
    let mac_lit = LitStr::new(&mac::section(val), Span::call_site());

    quote! {
        #[cfg_attr(all(target_os = "none",target_os = "linux",target_os = "android", target_os = "fuchsia",target_os = "freebsd" , target_os = "openbsd"), unsafe(link_section = #linux_lit))]
        #[cfg_attr(all(target_os = "windows",target_os = "uefi"),unsafe(link_section = #windows_lit))]
        #[cfg_attr(all(target_os = "macos" , target_os = "ios", target_os = "tvos"),unsafe(link_section = #mac_lit))]
        #item
    }
    .into()
}

#[proc_macro_attribute]
pub fn start(attr: TS, item: TS) -> TS {
    let attr = parse_macro_input!(attr as LitStr);
    let item = parse_macro_input!(item as Item);

    let val = &attr.value();

    if !val.is_ascii() {
        panic!("value must be an ASCII string")
    }

    let linux_lit = LitStr::new(&linux::section_start(val), Span::call_site());
    let windows_lit = LitStr::new(&windows::section_start(val), Span::call_site());
    let mac_lit = LitStr::new(&mac::section_start(val), Span::call_site());

    quote! {
        #[cfg_attr(all(target_os = "none", target_os = "linux" , target_os = "android" , target_os = "fuchsia",target_os = "freebsd", target_os = "openbsd"),link_name = #linux_lit)]
        #[cfg_attr(all(target_os = "uefi", target_os = "windows" ),link_name = #windows_lit)]
        #[cfg_attr(all(target_os = "macos", target_os = "ios", target_os = "tvos"),link_name = #mac_lit)]
        #item
    }.into()
}

#[proc_macro_attribute]
pub fn stop(attr: TS, item: TS) -> TS {
    let attr = parse_macro_input!(attr as LitStr);
    let item = parse_macro_input!(item as Item);

    let val = &attr.value();

    if !val.is_ascii() {
        panic!("value must be an ASCII string")
    }

    let linux_lit = LitStr::new(&linux::section_end(val), Span::call_site());
    let windows_lit = LitStr::new(&windows::section_end(val), Span::call_site());
    let mac_lit = LitStr::new(&mac::section_end(val), Span::call_site());

    quote! {
        #[cfg_attr(all(target_os = "none", target_os = "linux" , target_os = "android" , target_os = "fuchsia",target_os = "freebsd", target_os = "openbsd"),link_name = #linux_lit)]
        #[cfg_attr(all(target_os = "uefi", target_os = "windows" ),link_name = #windows_lit)]
        #[cfg_attr(all(target_os = "macos", target_os = "ios", target_os = "tvos"),link_name = #mac_lit)]
        #item
    }.into()
}

mod linux {
    pub fn section(input: &str) -> String {
        format!("linkeasy_{}", input)
    }
    pub fn section_start(input: &str) -> String {
        format!("__start_linkeasy_{}", input)
    }
    pub fn section_end(input: &str) -> String {
        format!("__stop_linkeasy_{}", input)
    }
}

mod windows {
    pub fn section(input: &str) -> String {
        format!(".linkeasy_{}$b", input)
    }
    pub fn section_start(input: &str) -> String {
        format!(".linkeasy_{}$a", input)
    }
    pub fn section_end(input: &str) -> String {
        format!(".linkeasy_{}$c", input)
    }
}

mod mac {
    use std::hash::{Hash, Hasher};

    fn hash_input(input: &str) -> String {
        let mut hasher = std::collections::hash_map::DefaultHasher::new();
        input.hash(&mut hasher);
        let mut remainder = hasher.finish();
        let mut out = String::new();
        for _ in 0..8 {
            let digit = (remainder % 62) as u8;
            remainder /= 62;
            out.push(match digit {
                0..=25 => (b'a' + digit) as char,
                26..=51 => (b'A' + digit - 26) as char,
                52..=61 => (b'0' + digit - 52) as char,
                _ => unreachable!(),
            });
        }
        out
    }
    pub fn section(input: &str) -> String {
        format!(
            "__DATA,__linkeasy{},regular,no_dead_strip",
            hash_input(input)
        )
    }
    pub fn section_start(input: &str) -> String {
        format!("\x01section$start$__DATA$__linkeasy{}", hash_input(input))
    }
    pub fn section_end(input: &str) -> String {
        format!("\x01section$end$__DATA$__linkeasy{}", hash_input(input))
    }
}
