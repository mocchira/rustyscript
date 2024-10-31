///
/// This example shows the definition of a simple extension exposing one op
/// Also see example_extension.js
///
/// Extensions like the one below allow you to let JS code call functions
/// in rust
///
/// Extensions consist of a set of #[op2] functions, an extension! macro,
/// and one or more optional JS modules.
///
///
use rustyscript::deno_core::{extension, op2};

#[op2(fast)]
#[bigint]
fn op_add_example(#[bigint] a: i64, #[bigint] b: i64) -> i64 {
    a + b
}

extension!(
    example_extension,
    ops = [op_add_example],
    esm_entry_point = "example:calculator",
    esm = [ dir "examples/example_extension", "example:calculator" = "example_extension.js" ],
);
