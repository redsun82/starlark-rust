/*
 * Copyright 2019 The Starlark in Rust Authors.
 * Copyright (c) Facebook, Inc. and its affiliates.
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     https://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

//! A proc-macro for writing functions in Rust that can be called from Starlark.

#![feature(box_patterns)]

#[allow(unused_extern_crates)] // proc_macro is very special
extern crate proc_macro;

use proc_macro::TokenStream;
use syn::*;

mod parse;
mod render;
mod trace;
mod typ;
mod util;

/// Write Starlark modules concisely in Rust syntax.
///
/// For example:
///
/// ```ignore
/// #[starlark_module]
/// fn global(registry: &mut GlobalsBuilder) {
///     fn cc_binary(name: &str, srcs: Vec<&str>) -> String {
///         Ok(format!("{:?} {:?}", name, srcs))
///     }
/// }
/// ```
///
/// Parameters operate as named parameters of a given type, with six possible tweaks:
///
/// * `this` (or `_this`) as the first argument means the argument is passed as a
///   bound method value, e.g. in `a.f(...)` the `a` would be `this`.
/// * `args` means the argument is the `*args`.
/// * `kwargs` means the argument is the `**kwargs`.
/// * `ref name` means the argument must be passed by position, not by name.
/// * A type of `Option` means the argument is optional.
/// * A pattern `x @ foo : bool` means the argument defaults to `foo` if not
///   specified.
///
/// During execution there are two local variables injected into scope:
///
/// * `eval` is the `Evaluator`.
/// * `heap` is the `Heap`, obtained from `eval.heap()`.
///
/// A function with the `#[starlark_module]` attribute can be added to a `GlobalsBuilder` value
/// using the `with` function. Those `Globals` can be passed to `Evaluator` to provide global functions.
/// Alternatively, you can return `Globals` from `get_methods` to _attach_ functions to
/// a specific type (e.g. the `string` type).
///
/// * When unattached, you can define constants with `const`. We define `True`, `False` and
///   `None` that way.
/// * When attached, you can annotate the functions with `#[attribute]` to turn the name into
///   an attribute on the value. Such a function must take exactly one argument, namely a value
///   of the type you have attached it to.
/// * The attribute `#[starlark_type("test")]` causes `f.type` to return `"test"`.
///
/// All these functions interoperate properly with `dir()`, `getattr()` and `hasattr()`.
///
/// If a desired function name is also a Rust keyword, use the `r#` prefix, e.g. `r#type`.
#[proc_macro_attribute]
pub fn starlark_module(attr: TokenStream, input: TokenStream) -> TokenStream {
    let input = parse_macro_input!(input as ItemFn);
    assert!(attr.is_empty());
    let mut x = parse::parse(input);
    x.resolve();
    render::render(x).into()
}

/// Derive the `Trace` trait.
#[proc_macro_derive(Trace)]
pub fn derive_trace(input: proc_macro::TokenStream) -> proc_macro::TokenStream {
    trace::derive_trace(input)
}
