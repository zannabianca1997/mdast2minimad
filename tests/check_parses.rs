/// Main test implementation, called for every test source in `sources`
fn test_source(source: &'static str) {
    // parsing the test with markdown
    let ast =
        markdown::to_mdast(&source, &Default::default()).expect("Markdown has no syntax errors");
    // convertint it
    if let Err(error) = mdast2minimad::to_minimad(&ast) {
        panic!("Cannot convert: {error}")
    }
}

/// Macro to create the test function for every source, and nested ones
macro_rules! tests {
    (
        $name:ident : $path:literal
        $( , $($more:tt)* )?
    ) => {
        #[test]
        fn $name() {
            crate::test_source(include_str!($path))
        }

        $(
            tests!{ $($more)* }
        )?
    };
    (
         mod $name:ident { $($inner:tt)* }
        $( , $($more:tt)* )?
    ) => {
        mod $name {
            tests! { $($inner)* }
        }

        $(
            tests!{ $($more)* }
        )?
    };
    () => {};
}

// Test sources.
// The list can be updated with the python script `update_sources.py`. The preceding and ending markers are used to find the index in the code.
// <test-index>
tests! {
    basic: "sources/basic.md",
}
// </test-index>
