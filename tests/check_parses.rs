use std::error::Error;

use mdast2minimad::{md_parse_options, to_minimad};

/// Main test implementation, called for every test source in `sources`
fn test_source(source: &'static str) {
    // parsing the test with markdown
    let ast =
        markdown::to_mdast(&source, &md_parse_options()).expect("Markdown has no syntax errors");
    // convertint it
    if let Err(error) = to_minimad(&ast) {
        eprintln!("{error}");
        if let Some(mut source) = error.source() {
            eprintln!();
            eprintln!("Cause:");
            eprintln!("  - {source}");
            while let Some(next_source) = source.source() {
                source = next_source;
                eprintln!("  - {source}");
            }
        }
        panic!("Error during conversion");
    }
}

include! {env!("TEST_SOURCES_RS")}
