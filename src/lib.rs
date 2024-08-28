#![doc = include_str!("../README.md")]

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;
use minimad::{Composite, CompositeStyle, Compound, Line};

#[derive(Clone, Debug, Display, Error)]
pub enum ToMinimadError {
    #[display("`{_0}` node is not supported")]
    UnsupportedNode(#[error(not(source))] &'static str),
}

/// Convert the markdown AST to a minimad Text
pub fn to_minimad(ast: &mdast::Node) -> Result<minimad::Text, ToMinimadError> {
    let mut emitter = Emitter::new(Options::default());
    emitter.node(ast)?;
    Ok(emitter.finish())
}

#[derive(Debug, Clone, Copy, Default)]
/// Options for the conversion
pub struct Options {}

/// Minimad code emitter
struct Emitter<'a> {
    /// Lines already emitted
    lines: Vec<minimad::Line<'a>>,
    /// Conversion options
    options: Options,
    /// If bold is currently setted
    bold: bool,
    /// If italic is currently setted
    italic: bool,
    /// If code is currently setted
    code: bool,
    /// If strikeout is currently setted
    strikeout: bool,
}
impl<'a> Emitter<'a> {
    /// Create a new, empty emitter
    fn new(options: Options) -> Self {
        Self {
            lines: vec![],
            options,
            bold: false,
            italic: false,
            code: false,
            strikeout: false,
        }
    }

    /// Emit a node
    fn node(&mut self, node: &'a mdast::Node) -> Result<(), ToMinimadError> {
        // emit the node
        match node {
            mdast::Node::Root(root) => self.root(root),
            mdast::Node::Heading(heading) => self.heading(heading),
            mdast::Node::Text(text) => self.text(text),
            mdast::Node::InlineCode(inline_code) => self.inline_code(inline_code),
            mdast::Node::BlockQuote(block_quote) => self.block_quote(block_quote),
            mdast::Node::FootnoteDefinition(footnote_definition) => {
                self.footnote_definition(footnote_definition)
            }
            mdast::Node::MdxJsxFlowElement(mdx_jsx_flow_element) => {
                self.mdx_jsx_flow_element(mdx_jsx_flow_element)
            }
            mdast::Node::List(list) => self.list(list),
            mdast::Node::MdxjsEsm(mdxjs_esm) => self.mdxjs_esm(mdxjs_esm),
            mdast::Node::Toml(toml) => self.toml(toml),
            mdast::Node::Yaml(yaml) => self.yaml(yaml),
            mdast::Node::Break(break_) => self.break_(break_),
            mdast::Node::InlineMath(inline_math) => self.inline_math(inline_math),
            mdast::Node::Delete(delete) => self.delete(delete),
            mdast::Node::Emphasis(emphasis) => self.emphasis(emphasis),
            mdast::Node::MdxTextExpression(mdx_text_expression) => {
                self.mdx_text_expression(mdx_text_expression)
            }
            mdast::Node::FootnoteReference(footnote_reference) => {
                self.footnote_reference(footnote_reference)
            }
            mdast::Node::Html(html) => self.html(html),
            mdast::Node::Image(image) => self.image(image),
            mdast::Node::ImageReference(image_reference) => self.image_reference(image_reference),
            mdast::Node::MdxJsxTextElement(mdx_jsx_text_element) => {
                self.mdx_jsx_text_element(mdx_jsx_text_element)
            }
            mdast::Node::Link(link) => self.link(link),
            mdast::Node::LinkReference(link_reference) => self.link_reference(link_reference),
            mdast::Node::Strong(strong) => self.strong(strong),
            mdast::Node::Code(code) => self.code(code),
            mdast::Node::Math(math) => self.math(math),
            mdast::Node::MdxFlowExpression(mdx_flow_expression) => {
                self.mdx_flow_expression(mdx_flow_expression)
            }
            mdast::Node::Table(table) => self.table(table),
            mdast::Node::ThematicBreak(thematic_break) => self.thematic_break(thematic_break),
            mdast::Node::TableRow(table_row) => self.table_row(table_row),
            mdast::Node::TableCell(table_cell) => self.table_cell(table_cell),
            mdast::Node::ListItem(list_item) => self.list_item(list_item),
            mdast::Node::Definition(definition) => self.definition(definition),
            mdast::Node::Paragraph(paragraph) => self.paragraph(paragraph),
        }
    }

    /// Emit a `Root` node
    fn root(
        &mut self,
        mdast::Root {
            position: _,
            children,
        }: &'a mdast::Root,
    ) -> Result<(), ToMinimadError> {
        for child in children {
            self.node(child)?;
        }
        Ok(())
    }

    /// Emit a `Heading` node
    fn heading(
        &mut self,
        mdast::Heading {
            position: _,
            children,
            depth,
        }: &'a mdast::Heading,
    ) -> Result<(), ToMinimadError> {
        self.lines.push(Line::Normal(Composite {
            style: CompositeStyle::Header(*depth),
            compounds: vec![],
        }));
        for child in children {
            self.node(child)?;
        }
        Ok(())
    }

    /// Emit a `Text` node
    fn text(
        &mut self,
        mdast::Text { position: _, value }: &'a mdast::Text,
    ) -> Result<(), ToMinimadError> {
        let compound = Compound {
            src: &value,
            bold: self.bold,
            italic: self.italic,
            code: self.code,
            strikeout: self.strikeout,
        };
        self.current_compound_line().push(compound);
        Ok(())
    }

    /// Emit a `InlineCode` node
    fn inline_code(
        &mut self,
        mdast::InlineCode { position: _, value }: &'a mdast::InlineCode,
    ) -> Result<(), ToMinimadError> {
        let compound = Compound {
            src: &value,
            bold: self.bold,
            italic: self.italic,
            code: true,
            strikeout: self.strikeout,
        };
        self.current_compound_line().push(compound);
        Ok(())
    }

    /// Emit a `BlockQuote` node
    fn block_quote(
        &mut self,
        mdast::BlockQuote { position: _, .. }: &'a mdast::BlockQuote,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("BlockQuote"))
    }

    /// Emit a `FootnoteDefinition` node
    fn footnote_definition(
        &mut self,
        mdast::FootnoteDefinition { position: _, .. }: &'a mdast::FootnoteDefinition,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("FootnoteDefinition"))
    }

    /// Emit a `MdxJsxFlowElement` node
    fn mdx_jsx_flow_element(
        &mut self,
        mdast::MdxJsxFlowElement { position: _, .. }: &'a mdast::MdxJsxFlowElement,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("MdxJsxFlowElement"))
    }

    /// Emit a `List` node
    fn list(
        &mut self,
        mdast::List { position: _, .. }: &'a mdast::List,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("List"))
    }

    /// Emit a `MdxjsEsm` node
    fn mdxjs_esm(
        &mut self,
        mdast::MdxjsEsm { position: _, .. }: &'a mdast::MdxjsEsm,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("MdxjsEsm"))
    }

    /// Emit a `Toml` node
    fn toml(
        &mut self,
        mdast::Toml { position: _, .. }: &'a mdast::Toml,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Toml"))
    }

    /// Emit a `Yaml` node
    fn yaml(
        &mut self,
        mdast::Yaml { position: _, .. }: &'a mdast::Yaml,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Yaml"))
    }

    /// Emit a `Break` node
    fn break_(
        &mut self,
        mdast::Break { position: _, .. }: &'a mdast::Break,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Break"))
    }

    /// Emit a `InlineMath` node
    fn inline_math(
        &mut self,
        mdast::InlineMath { position: _, .. }: &'a mdast::InlineMath,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("InlineMath"))
    }

    /// Emit a `Delete` node
    fn delete(
        &mut self,
        mdast::Delete { position: _, .. }: &'a mdast::Delete,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Delete"))
    }

    /// Emit a `Emphasis` node
    fn emphasis(
        &mut self,
        mdast::Emphasis { position: _, .. }: &'a mdast::Emphasis,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Emphasis"))
    }

    /// Emit a `MdxTextExpression` node
    fn mdx_text_expression(
        &mut self,
        mdast::MdxTextExpression { position: _, .. }: &'a mdast::MdxTextExpression,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("MdxTextExpression"))
    }

    /// Emit a `FootnoteReference` node
    fn footnote_reference(
        &mut self,
        mdast::FootnoteReference { position: _, .. }: &'a mdast::FootnoteReference,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("FootnoteReference"))
    }

    /// Emit a `Html` node
    fn html(
        &mut self,
        mdast::Html { position: _, .. }: &'a mdast::Html,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Html"))
    }

    /// Emit a `Image` node
    fn image(
        &mut self,
        mdast::Image { position: _, .. }: &'a mdast::Image,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Image"))
    }

    /// Emit a `ImageReference` node
    fn image_reference(
        &mut self,
        mdast::ImageReference { position: _, .. }: &'a mdast::ImageReference,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("ImageReference"))
    }

    /// Emit a `MdxJsxTextElement` node
    fn mdx_jsx_text_element(
        &mut self,
        mdast::MdxJsxTextElement { position: _, .. }: &'a mdast::MdxJsxTextElement,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("MdxJsxTextElement"))
    }

    /// Emit a `Link` node
    fn link(
        &mut self,
        mdast::Link { position: _, .. }: &'a mdast::Link,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Link"))
    }

    /// Emit a `LinkReference` node
    fn link_reference(
        &mut self,
        mdast::LinkReference { position: _, .. }: &'a mdast::LinkReference,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("LinkReference"))
    }

    /// Emit a `Strong` node
    fn strong(
        &mut self,
        mdast::Strong { position: _, .. }: &'a mdast::Strong,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Strong"))
    }

    /// Emit a `Code` node
    fn code(
        &mut self,
        mdast::Code {
            position: _,
            value,
            lang: _,
            meta: _,
        }: &'a mdast::Code,
    ) -> Result<(), ToMinimadError> {
        for line in value.lines() {
            self.lines.push(Line::Normal(Composite {
                style: CompositeStyle::Code,
                compounds: vec![Compound {
                    src: line,
                    bold: false,
                    italic: false,
                    code: false,
                    strikeout: false,
                }],
            }))
        }
        Ok(())
    }

    /// Emit a `Math` node
    fn math(
        &mut self,
        mdast::Math { position: _, .. }: &'a mdast::Math,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Math"))
    }

    /// Emit a `MdxFlowExpression` node
    fn mdx_flow_expression(
        &mut self,
        mdast::MdxFlowExpression { position: _, .. }: &'a mdast::MdxFlowExpression,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("MdxFlowExpression"))
    }

    /// Emit a `Table` node
    fn table(
        &mut self,
        mdast::Table { position: _, .. }: &'a mdast::Table,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Table"))
    }

    /// Emit a `ThematicBreak` node
    fn thematic_break(
        &mut self,
        mdast::ThematicBreak { position: _, .. }: &'a mdast::ThematicBreak,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("ThematicBreak"))
    }

    /// Emit a `TableRow` node
    fn table_row(
        &mut self,
        mdast::TableRow { position: _, .. }: &'a mdast::TableRow,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("TableRow"))
    }

    /// Emit a `TableCell` node
    fn table_cell(
        &mut self,
        mdast::TableCell { position: _, .. }: &'a mdast::TableCell,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("TableCell"))
    }

    /// Emit a `ListItem` node
    fn list_item(
        &mut self,
        mdast::ListItem { position: _, .. }: &'a mdast::ListItem,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("ListItem"))
    }

    /// Emit a `Definition` node
    fn definition(
        &mut self,
        mdast::Definition { position: _, .. }: &'a mdast::Definition,
    ) -> Result<(), ToMinimadError> {
        Err(ToMinimadError::UnsupportedNode("Definition"))
    }

    /// Emit a `Paragraph` node
    fn paragraph(
        &mut self,
        mdast::Paragraph {
            position: _,
            children,
        }: &'a mdast::Paragraph,
    ) -> Result<(), ToMinimadError> {
        // generate a new line
        self.lines.push(Line::Normal(Composite {
            style: CompositeStyle::Paragraph,
            compounds: vec![],
        }));
        for child in children {
            self.node(child)?;
        }
        Ok(())
    }

    /// Complete the emission
    fn finish(self) -> minimad::Text<'a> {
        minimad::Text { lines: self.lines }
    }

    /// Get the last line where to push inline elements
    fn current_compound_line(&mut self) -> &mut Vec<Compound<'a>> {
        if !matches!(
            self.lines.last(),
            Some(Line::Normal(_) | Line::CodeFence(_)),
        ) {
            self.lines.push(Line::Normal(Composite {
                style: CompositeStyle::Paragraph,
                compounds: vec![],
            }));
        }
        let Some(
            Line::Normal(Composite { compounds, .. })
            | Line::CodeFence(Composite { compounds, .. }),
        ) = self.lines.last_mut()
        else {
            unreachable!()
        };
        compounds
    }
}
