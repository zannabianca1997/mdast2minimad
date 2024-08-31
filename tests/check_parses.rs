use markdown::ParseOptions;

/// Main test implementation, called for every test source in `sources`
fn test_source(source: &'static str) {
    // parsing the test with markdown
    let ast =
        markdown::to_mdast(&source, &ParseOptions::gfm()).expect("Markdown has no syntax errors");
    // convertint it
    if let Err(error) = mdast2minimad::to_minimad(&ast) {
        panic!("Cannot convert: {error}")
    }
}

include! {env!("TEST_SOURCES_RS")}
