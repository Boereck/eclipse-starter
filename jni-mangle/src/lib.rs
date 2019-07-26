/*******************************************************************************
 * Copyright (c) 2019 Fraunhofer FOKUS and others.
 *
 * This program and the accompanying materials
 * are made available under the terms of the Eclipse Public License 2.0
 * which accompanies this distribution, and is available at 
 * https://www.eclipse.org/legal/epl-2.0/
 *
 * SPDX-License-Identifier: EPL-2.0
 * 
 * Contributors:
 *     Max Bureck (Fraunhofer FOKUS)
 *******************************************************************************/

//! This crate provides the proc macro `jni_mangle`, this can be applied 
//! to method to mangle their namse so they can be called via JNI.

extern crate proc_macro;

use syn::{parse_macro_input, DeriveInput, LitStr, ItemFn, Ident};
use quote::quote;
use crate::proc_macro::TokenStream;
 
/// This macro is used to mangle names of functions, so they can be called from JNI.
/// The attribute takes the fully qualified name of a Java class,
/// such as `#[jni_mangle("foo.baz.Bar")]` which will be used 
/// for mangling the function name.
#[proc_macro_attribute]
pub fn jni_mangle(attr: TokenStream, item: TokenStream) -> TokenStream {
    let ast: LitStr = parse_macro_input!(attr as LitStr);
    let java_class = ast.value();
    let mut annotated_fn = parse_macro_input!(item as ItemFn);
    let fn_name = annotated_fn.ident.to_string();
    let mangled_name = mangle_name(&java_class, &fn_name);
    let old_ident = annotated_fn.ident;
    annotated_fn.ident = Ident::new(&mangled_name, old_ident.span());
    quote!(#annotated_fn).into()
}

fn mangle_name(java_class: &str, fn_name: &str) -> String {
    let java_class = java_class.replace(".", "_");
    let fn_name = fn_name.replace("_", "_1");
    format!("Java_{}__1{}", java_class, fn_name)
}