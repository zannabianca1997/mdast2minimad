#!/bin/env python3
"""
This is an helper script to update the list of sources

The build script is not rerun for test sources, so it is not able to help in generating the multiple test functions
"""
from pathlib import Path
from glob import glob
import re
from typing import Union


def main():
    # find the sources dir
    sources_dir = Path(__file__).parent.joinpath("sources")
    # find the file to modify
    test_file = Path(__file__).parent.joinpath("check_parses.rs")
    # update the file
    update_test_file(sources_dir, test_file)


def rustify(name: str) -> str:
    """Slugify a name into a rust identifier"""
    return name


Index = dict[str, Union[Path, "Index"]]


def update_test_file(sources_dir: Path, test_file: Path):
    """Update the test file with the current list of sources"""
    # building up the index of all sources
    index: Index = {}
    for source in glob("**/*.md", root_dir=sources_dir, recursive=True):
        # reconstruct the full path from the test file
        source_path = sources_dir.joinpath(source).relative_to(test_file.parent)
        # finding the rust name of this test
        rust_name = rustify(source_path.stem)
        # finding the rust path of this test
        rust_path = [rustify(part) for part in Path(source).parent.parts]
        # advising the user about our choiche
        print(f"Adding {'::'.join((*rust_path, rust_name))} from {source_path}")
        # inserting the item in the index
        dest = index
        for part in rust_path:
            if part not in dest:
                dest[part] = {}
            dest = dest[part]
        dest[rust_name] = source_path
    # building the rust macro input
    rust_macro_input = "\n".join(
        macro_input_item(name, item) for name, item in index.items()
    )
    rust_macro_call = f"tests! {{\n{rust_macro_input}\n}}"
    # reading the current file content
    test_file_initial_content = test_file.read_text()
    # substitute the macro calls
    test_file_final_content = re.sub(
        r"(?m)^\s*//\s*<\s*test-index\s*>\s*$(?:.|\n|\r)*^\s*//\s*<\s*/\s*test-index\s*>\s*$",
        f"// <test-index>\n{rust_macro_call}\n// </test-index>",
        test_file_initial_content,
    )
    # write it back
    test_file.write_text(test_file_final_content)


def macro_input_item(name: str, item: Union[Path, Index], *, indent: int = 1) -> str:
    """Build the macro input item"""
    if isinstance(item, dict):
        inner = "\n".join(
            macro_input_item(name, item, indent=indent + 1)
            for name, item in item.items()
        )
        return "    " * indent + f"mod {name} {{\n{inner}\n" + "    " * indent + "},"
    else:
        return "    " * indent + f'{name}: "{item}",'


if __name__ == "__main__":
    main()
