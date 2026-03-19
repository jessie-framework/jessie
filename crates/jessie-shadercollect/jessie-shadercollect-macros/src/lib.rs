use std::{ffi::CString, str::FromStr};

use proc_macro2::Literal;
use quote::quote;
use syn::{DeriveInput, parse_macro_input};

type TS = proc_macro::TokenStream;
type TS2 = proc_macro2::TokenStream;

#[proc_macro_derive(IntoAttrPtr)]
pub fn into_locations(item: TS) -> TS {
    let item = parse_macro_input!(item as DeriveInput);
    let idt = &item.ident;
    let generics = &item.generics;
    let mut attr_count: usize = 0;
    let mut handles: Vec<TS2> = vec![];
    match &item.data {
        syn::Data::Struct(stct) => match &stct.fields {
            syn::Fields::Named(fields) => {
                for field in &fields.named {
                    attr_count += 1;
                    let ident = field.ident.clone().unwrap();
                    let ty = field.ty.clone();
                    let name = Literal::c_string(
                        CString::from_str(&ident.to_string()).unwrap().as_c_str(),
                    );
                    handles.push(quote! {
                        jessie_shadercollect::prelude::LocationHandle {
                            name : #name,
                            offset : std::mem::offset_of!(Self,#ident) as i32,
                            gl_type : <#ty as jessie_shadercollect::prelude::IntoGlType>::GL_TYPE,
                            size : <#ty as jessie_shadercollect::prelude::IntoGlSize>::GL_SIZE,


                        }
                    });
                }
            }

            syn::Fields::Unnamed(fields) => {
                for field in &fields.unnamed {
                    attr_count += 1;
                    let ty = field.ty.clone();
                    let lit = Literal::usize_unsuffixed(attr_count);
                    let name = Literal::c_string(
                        &CString::from_str(&format!("attr{}", attr_count)).unwrap(),
                    );
                    handles.push(quote! {
                        jessie_shadercollect::prelude::LocationHandle {
                            name : #name,
                            offset : std::mem::offset_of!(Self,#lit) as i32,
                            gl_type : <#ty as jessie_shadercollect::prelude::IntoGlType>::GL_TYPE,
                            size : <#ty as jessie_shadercollect::prelude::IntoGlSize>::GL_SIZE
                        }
                    });
                }
            }
            _ => unreachable!("Item can not be empty"),
        },
        _ => unreachable!("Item must be a struct"),
    }
    let attr_lit = proc_macro2::Literal::usize_suffixed(attr_count);
    quote! {
        unsafe impl jessie_shadercollect::prelude::IntoAttrPtr for #idt #generics {
            const ATTRPTR : jessie_shadercollect::prelude::AttrPtrHandle = jessie_shadercollect::prelude::AttrPtrHandle {
              stride : std::mem::size_of::<Self>() as i32,
              attr_count : #attr_lit,
              locs : &[
                  #(#handles),*
              ]  
            };

        }
    }
    .into()
}

