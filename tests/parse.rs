use test_generator::test_resources;

/// Main test implementation, called for every test source in `sources`
#[test_resources("md_sources/**/*.md")]
fn parse(source: &'static str) {
    // parsing the test with markdown
    let ast =
        markdown::to_mdast(&source, &Default::default()).expect("Markdown has no syntax errors");
    // convertint it
    if let Err(error) = mdast2minimad::to_minimad(&ast) {
        panic!("Cannot convert: {error}")
    }
}
