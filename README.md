# `mdast2minimad`
This crate has the objective to export the largest possible subset of [mdast](https://github.com/syntax-tree/mdast) from the crate [markdown](https://github.com/wooorm/markdown-rs) to the ast exposed by the crate [minimad](https://docs.rs/minimad/latest/minimad/index.html). 

The main use of this crate is in [dices](https://github.com/zannabianca1997/dices), where the ast is edited to render the code example before using [termimand](https://github.com/Canop/termimad) to print them.

## Limitations
[minimad](https://docs.rs/minimad/latest/minimad/index.html) parser is not a fully fledget markdown parser. This make the two ASTs impossble to transpose: for example this two snippets of markdown represent the same document, but are parsed differently by [minimad](https://docs.rs/minimad/latest/minimad/index.html):
```markdown
This is a paragraph.

This is another.
```
```markdown
This is a paragraph.



This is another.
```
The translation is then made on a _best effort_ base.

## License
This software is distributed under the **MIT** license, if you need to know. Use it at will.

## Contacts
This library was made by *zannabianca1997*.
If you found any problem with the library itself, or want to contribute, you can send a PR, or contact me at [zannabianca199712@gmail.com](mailto:zannabianca199712@gmail.com).