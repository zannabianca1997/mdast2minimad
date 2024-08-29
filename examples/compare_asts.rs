//! This example compare the two asts generated by direct `minimad` parsing and conversion from `markdown`->`minimad`

use std::{
    ffi::{OsStr, OsString},
    fs,
    path::{Path, PathBuf},
};

use anyhow::{Context, Result};
use clap::{Args, Parser};

#[derive(Debug, Parser)]
#[command(version = "0.1.0", name = "compare_asts")]
/// Compare the two asts generated by direct `minimad` parsing and conversion from `markdown`->`minimad`
struct Cli {
    /// Input markdown file
    markdown: PathBuf,
    /// Output of the direct `minimad` parsing
    minimad: Option<PathBuf>,
    /// Output of the `markdown` parsed ast
    parsed: Option<PathBuf>,
    /// Output of the converted ast
    converted: Option<PathBuf>,

    #[clap(flatten)]
    mm_opt: MinimadOptions,
}

#[derive(Debug, Clone, Copy, Args)]
/// Markdown parsing options for `minimad`
struct MinimadOptions {
    #[clap(long, short = 'c')]
    continue_inline_code: bool,
    #[clap(long, short = 'i')]
    continue_italic: bool,
    #[clap(long, short = 'b')]
    continue_bold: bool,
    #[clap(long, short = 's')]
    continue_strikeout: bool,
}
impl MinimadOptions {
    fn build(self) -> minimad::Options {
        let Self {
            continue_inline_code,
            continue_italic,
            continue_bold,
            continue_strikeout,
        } = self;
        minimad::Options {
            clean_indentations: minimad::Options::default().clean_indentations,
            continue_inline_code,
            continue_italic,
            continue_bold,
            continue_strikeout,
        }
    }
}

fn main() -> Result<()> {
    let Cli {
        markdown,
        minimad,
        parsed,
        converted,
        mm_opt,
    } = Cli::parse();
    let minimad_file = minimad.unwrap_or_else(|| markdown.with_extension("minimad"));
    let parsed_file = parsed.unwrap_or_else(|| markdown.with_extension("markdown"));
    let converted_file = converted.unwrap_or_else(|| markdown.with_extension("converted"));
    let mm_opt = mm_opt.build();

    // read the sources
    let markdown = fs::read_to_string(markdown).context("Cannot read input file")?;

    // Direct minimad parsing
    let parsed_mm = minimad::parse_text(&markdown, mm_opt);
    fs::write(minimad_file, format!("{parsed_mm:#?}")).context("Cannot write to minimad file")?;

    // Conversion
    let parsed_md = markdown::to_mdast(&markdown, &Default::default())
        .expect("Pure markdown has no syntax errors");
    fs::write(parsed_file, format!("{parsed_md:#?}")).context("Cannot write to parsed file")?;
    let converted = mdast2minimad::to_minimad(&parsed_md)
        .map_err(|err| err.into_static())
        .context("Cannot convert the markdown")?;
    fs::write(converted_file, format!("{converted:#?}"))
        .context("Cannot write to converted file")?;

    if parsed_mm == converted {
        println!("Full success! The asts are identical")
    } else {
        println!("There are some differences in the asts")
    }

    Ok(())
}
