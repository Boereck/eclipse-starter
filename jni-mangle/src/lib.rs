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
/// for mangling the function name. Note that the macro assumes the 
/// Java method stars with a "_" prefix, whereas the Rust method does not.
#[proc_macro_attribute]
pub fn jni_mangle(attr: TokenStream, item: TokenStream) -> TokenStream {
    // Parse input
    let ast: LitStr = parse_macro_input!(attr as LitStr);
    let mut annotated_fn = parse_macro_input!(item as ItemFn);

    // Get info for name mangling
    let java_class = ast.value();
    let fn_name = annotated_fn.ident.to_string();

    // Adjust function name to mangled
    let mangled_name = mangle_name(&java_class, &fn_name);
    let old_ident = annotated_fn.ident;
    annotated_fn.ident = Ident::new(&mangled_name, old_ident.span());

    // return modified function
    quote!(#annotated_fn).into()
}

/// Applies name mangling rules according to 
/// [JNI spec](https://docs.oracle.com/javase/8/docs/technotes/guides/jni/spec/design.html#resolving_native_method_names)
/// but only to the extend needed by the `eclipse-library` project. Rules that
/// do not apply for names are omitted. Also note that this method
/// assumes that the Java method begins with an "_" prefix, whereas
/// the Rust method does not.
fn mangle_name(java_class: &str, fn_name: &str) -> String {
    let java_class = java_class.replace(".", "_");
    let fn_name = fn_name.replace("_", "_1");
    // Note: the separator between class and function name is usually
    // "__", but the original Java methods start with an "_", so we 
    // need the "1" prefix before the method name
    format!("Java_{}__1{}", java_class, fn_name)
}