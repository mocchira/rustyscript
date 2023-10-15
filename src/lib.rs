//! ## Effortless JS Integration for Rust
//! ### deno_core API wrapper that provides an easier way to call JS from rust, by abstracting away the v8 engine details
//!
//! [![Crates.io](https://img.shields.io/crates/v/js-playground.svg)](https://crates.io/crates/js-playground)
//! [![Build Status](https://github.com/rscarson/js-playground/workflows/Rust/badge.svg)](https://github.com/rscarson/js-playground/actions?workflow=Rust)
//! [![License](https://img.shields.io/badge/license-MIT-blue.svg)](https://raw.githubusercontent.com/rscarson/js-playground/master/LICENSE)
//!
//! This crate is meant to provide a quick and simple way to integrate a runtime javacript or typescript component from within rust.
//!
//! **By default, the code being run is entirely sandboxed from the host, having no filesystem or network access.**
//! It can be extended to include those capabilities and more if desired - please see the `runtime_extensions` example
//!
//! Asynchronous code is supported (I suggest using the timeout option when creating your runtime)
//! Loaded JS modules can import other modules
//!
//! ----
//!
//! Here is a very basic use of this crate to execute a JS module. It will create a basic runtime, load the module,
//! call the registered entrypoint function with the given arguments, and return the resulting value:
//! ```rust
//! use rustyscript::{json_args, Runtime, Module, Error};
//!
//! # fn main() -> Result<(), Error> {
//! let module = Module::new(
//!     "test.js",
//!     "
//!     rustyscript.register_entrypoint(
//!         (string, integer) => {
//!             console.log(`Hello world: string=${string}, integer=${integer}`);
//!             return 2;
//!         }
//!     )
//!     "
//! );
//!
//! let value: usize = Runtime::execute_module(
//!     &module, vec![],
//!     Default::default(),
//!     json_args!("test", 5)
//! )?;
//!
//! assert_eq!(value, 2);
//! # Ok(())
//! # }
//! ```
//!
//! Modules can also be loaded from the filesystem with `Module::load` or `Module::load_dir` if you want to collect all modules in a given directory.
//!
//! ----
//!
//! If all you need is the result of a single javascript expression, you can use:
//! ```rust
//! let result: i64 = rustyscript::evaluate("5 + 5").expect("The expression was invalid!");
//! ```
//!
//! Or to just import a single module for use:
//! ```no_run
//! use rustyscript::{json_args, import};
//! let mut module = import("js/my_module.js").expect("Something went wrong!");
//! let value: String = module.call("exported_function_name", json_args!()).expect("Could not get a value!");
//! ```
//!
//! There are a few other utilities included, such as `rustyscript::validate` and `rustyscript::resolve_path`
//!
//! ----
//!
//! A more detailed version of the crate's usage can be seen below, which breaks down the steps instead of using the one-liner `Runtime::execute_module`:
//! ```rust
//! use rustyscript::{json_args, Runtime, RuntimeOptions, Module, Error, Undefined};
//! use std::time::Duration;
//!
//! # fn main() -> Result<(), Error> {
//! let module = Module::new(
//!     "test.js",
//!     "
//!     let internalValue = 0;
//!     export const load = (value) => internalValue = value;
//!     export const getValue = () => internalValue;
//!     "
//! );
//!
//! // Create a new runtime
//! let mut runtime = Runtime::new(RuntimeOptions {
//!     timeout: Duration::from_millis(50), // Stop execution by force after 50ms
//!     default_entrypoint: Some("load".to_string()), // Run this as the entrypoint function if none is registered
//!     ..Default::default()
//! })?;
//!
//! // The handle returned is used to get exported functions and values from that module.
//! // We then call the entrypoint function, but do not need a return value.
//! //Load can be called multiple times, and modules can import other loaded modules
//! // Using `import './filename.js'`
//! let module_handle = runtime.load_module(&module)?;
//! runtime.call_entrypoint::<Undefined>(&module_handle, json_args!(2))?;
//!
//! let internal_value: i64 = runtime.call_function(&module_handle, "getValue", json_args!())?;
//! # Ok(())
//! # }
//! ```
//!
//! ----
//!
//! ## Utility Functions
//! These functions provide simple one-liner access to common features of this crate:
//! - evaluate; Evaluate a single JS expression and return the resulting value
//! - import; Get a handle to a JS module from which you can get exported values and functions
//! - resolve_path; Resolve a relative path to the current working dir
//! - validate; Validate the syntax of a JS expression
//!
//! ## Crate features
//! - console (deno_console); Add the deno_console crate, providing `console.*` functionality from JS
//! - url (deno_url, deno_webidl); Provides the WebIDL, URL, and URLPattern APIs from within JS
//! - web = (deno_webidl, deno_web, deno_crypto, deno_fetch); Provides the Event, TextEncoder, TextDecoder, File, Web Cryptography, and fetch APIs from within JS
//! - default (console, url); Provides only those extensions that preserve sandboxing between the host and runtime
//! - no_extensions; Disable all optional extensions to the runtime
//! - all (console, url, web)
//!
//! Please also check out [@Bromeon/js_sandbox](https://github.com/Bromeon/js-sandbox), another great crate in this niche
//!
//! For an example of this crate in use, please check out [lavendeux-parser](https://github.com/rscarson/lavendeux-parser)
//!
#![warn(missing_docs)]

mod error;
mod ext;
mod inner_runtime;
mod js_function;
mod module;
mod module_handle;
mod module_wrapper;
mod runtime;
mod traits;
mod transpiler;
mod utilities;

// Expose a few dependencies that could be useful
pub use deno_core;
pub use deno_core::serde_json;

// Expose some important stuff from us
pub use error::Error;
pub use inner_runtime::FunctionArguments;
pub use js_function::JsFunction;
pub use module::{Module, StaticModule};
pub use module_handle::ModuleHandle;
pub use module_wrapper::ModuleWrapper;
pub use runtime::{Runtime, RuntimeOptions, Undefined};
pub use utilities::{evaluate, import, resolve_path, validate};
