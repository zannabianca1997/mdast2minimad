//! This example compare the two asts generated by direct `minimad` parsing and conversion from `markdown`->`minimad`

use std::{fs, path::PathBuf};

use anyhow::{Context, Result};
use clap::Parser;

#[derive(Debug, Parser)]
#[command(version = "0.1.0", name = "display")]
/// Compare the two asts generated by direct `minimad` parsing and conversion from `markdown`->`minimad`
struct Cli {
    /// Input markdown file
    markdown: PathBuf,
    /// Use minimad as a parser, not markdown
    #[clap(long, short)]
    minimad: bool,
    /// Print the generated ASTs
    #[clap(long = "ast", short = 'a')]
    print_ast: bool,
}

fn main() -> Result<()> {
    let Cli {
        markdown,
        minimad,
        print_ast,
    } = Cli::parse();

    // read the sources
    let src = fs::read_to_string(markdown).context("Cannot read input file")?;

    let text = if minimad {
        // Parse with `minimad`
        minimad::parse_text(&src, minimad::Options::default())
    } else {
        // Parse with `markdown`
        let ast = markdown::to_mdast(&src, &markdown::ParseOptions::default())
            .expect("Markdown has no syntax errors");
        // Leak it: the ast must live until the print, and then the program will end.
        // There is no merit in keeping track of the AST lifetime
        let ast = &*Box::leak(Box::new(ast));

        if print_ast {
            println!("{:#?}", ast)
        }
        // Using our converter
        mdast2minimad::to_minimad(&ast).context("Error during ast conversion")?
    };

    if print_ast {
        println!("{:#?}", text)
    }

    // Display with `termimad`
    let formatted = termimad::FmtText::from_text(
        termimad::get_default_skin(),
        text,
        Some(termimad::terminal_size().0 as _),
    );
    print!("{formatted}");

    Ok(())
}
