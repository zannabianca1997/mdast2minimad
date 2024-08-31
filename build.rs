use std::{
    collections::HashMap,
    env, fs,
    path::{Path, PathBuf},
};

use proc_macro2::TokenStream;
use quote::{format_ident, quote};
use slugify::slugify;

const TEST_SOURCES_DIR: &str = "tests/sources";

fn main() {
    // scan the test sources to add them to the tests
    add_test_sources()
}

type TestSourcesDir = HashMap<String, TestSourcesItem>;
#[derive(Debug, Clone)]
enum TestSourcesItem {
    Dir(TestSourcesDir),
    Src(String),
}

fn add_test_sources() {
    // find and catalog all sources
    let sources = scan_test_sources_dir(Path::new(TEST_SOURCES_DIR));
    // create the tests
    let tests = make_tests(sources);
    // write them out to a file
    let out_file_path = PathBuf::from(env::var_os("OUT_DIR").unwrap()).join("test_sources.rs");
    println!(
        "cargo::rustc-env=TEST_SOURCES_RS={}",
        out_file_path.display()
    );
    fs::write(out_file_path, tests.to_string()).unwrap();
}

fn make_tests(dir: TestSourcesDir) -> TokenStream {
    let items = dir.into_iter().map(|(name, item)| {
        let name = format_ident!("{}", slugify!(&name, separator = "_"));
        match item {
            TestSourcesItem::Dir(dir) => {
                let inner = make_tests(dir);
                quote! {
                    mod #name {
                        #inner
                    }
                }
            }
            TestSourcesItem::Src(src) => {
                quote! {
                    #[test]
                    fn #name() {
                        crate::test_source(#src)
                    }
                }
            }
        }
    });
    quote! {
        #(#items)*
    }
}

fn scan_test_sources_dir(dir: &Path) -> TestSourcesDir {
    println!("cargo::rerun-if-changed={}", dir.display());
    let mut items = TestSourcesDir::new();
    for item in dir.read_dir().unwrap() {
        let item = item.unwrap();
        let name = item
            .path()
            .file_stem()
            .unwrap()
            .to_string_lossy()
            .into_owned();
        items.insert(
            name,
            if item.file_type().unwrap().is_file() {
                let path = item.path();
                println!("cargo::rerun-if-changed={}", path.display());
                TestSourcesItem::Src(fs::read_to_string(path).unwrap())
            } else {
                TestSourcesItem::Dir(scan_test_sources_dir(&item.path()))
            },
        );
    }
    items
}
