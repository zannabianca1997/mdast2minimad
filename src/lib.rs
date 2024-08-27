#![doc = include_str!("../README.md")]

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;
use markdown::unist::Position;

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
}
impl<'a> Emitter<'a> {
    /// Create a new, empty emitter
    fn new(options: Options) -> Self {
        Self {
            lines: vec![],
            options,
        }
    }

    /// Emit a node
    fn node(&mut self, node: &'a mdast::Node) -> Result<(), ToMinimadError> {
        // emit the node
        match node {
            mdast::Node::Root(root) => self.root(root),
            // -- Unsupported node types --
            mdast::Node::BlockQuote(_) => Err(ToMinimadError::UnsupportedNode("BlockQuote")),
            mdast::Node::FootnoteDefinition(_) => {
                Err(ToMinimadError::UnsupportedNode("FootnoteDefinition"))
            }
            mdast::Node::MdxJsxFlowElement(_) => {
                Err(ToMinimadError::UnsupportedNode("MdxJsxFlowElement"))
            }
            mdast::Node::List(_) => Err(ToMinimadError::UnsupportedNode("List")),
            mdast::Node::MdxjsEsm(_) => Err(ToMinimadError::UnsupportedNode("MdxjsEsm")),
            mdast::Node::Toml(_) => Err(ToMinimadError::UnsupportedNode("Toml")),
            mdast::Node::Yaml(_) => Err(ToMinimadError::UnsupportedNode("Yaml")),
            mdast::Node::Break(_) => Err(ToMinimadError::UnsupportedNode("Break")),
            mdast::Node::InlineCode(_) => Err(ToMinimadError::UnsupportedNode("InlineCode")),
            mdast::Node::InlineMath(_) => Err(ToMinimadError::UnsupportedNode("InlineMath")),
            mdast::Node::Delete(_) => Err(ToMinimadError::UnsupportedNode("Delete")),
            mdast::Node::Emphasis(_) => Err(ToMinimadError::UnsupportedNode("Emphasis")),
            mdast::Node::MdxTextExpression(_) => {
                Err(ToMinimadError::UnsupportedNode("MdxTextExpression"))
            }
            mdast::Node::FootnoteReference(_) => {
                Err(ToMinimadError::UnsupportedNode("FootnoteReference"))
            }
            mdast::Node::Html(_) => Err(ToMinimadError::UnsupportedNode("Html")),
            mdast::Node::Image(_) => Err(ToMinimadError::UnsupportedNode("Image")),
            mdast::Node::ImageReference(_) => {
                Err(ToMinimadError::UnsupportedNode("ImageReference"))
            }
            mdast::Node::MdxJsxTextElement(_) => {
                Err(ToMinimadError::UnsupportedNode("MdxJsxTextElement"))
            }
            mdast::Node::Link(_) => Err(ToMinimadError::UnsupportedNode("Link")),
            mdast::Node::LinkReference(_) => Err(ToMinimadError::UnsupportedNode("LinkReference")),
            mdast::Node::Strong(_) => Err(ToMinimadError::UnsupportedNode("Strong")),
            mdast::Node::Text(_) => Err(ToMinimadError::UnsupportedNode("Text")),
            mdast::Node::Code(_) => Err(ToMinimadError::UnsupportedNode("Code")),
            mdast::Node::Math(_) => Err(ToMinimadError::UnsupportedNode("Math")),
            mdast::Node::MdxFlowExpression(_) => {
                Err(ToMinimadError::UnsupportedNode("MdxFlowExpression"))
            }
            mdast::Node::Heading(_) => Err(ToMinimadError::UnsupportedNode("Heading")),
            mdast::Node::Table(_) => Err(ToMinimadError::UnsupportedNode("Table")),
            mdast::Node::ThematicBreak(_) => Err(ToMinimadError::UnsupportedNode("ThematicBreak")),
            mdast::Node::TableRow(_) => Err(ToMinimadError::UnsupportedNode("TableRow")),
            mdast::Node::TableCell(_) => Err(ToMinimadError::UnsupportedNode("TableCell")),
            mdast::Node::ListItem(_) => Err(ToMinimadError::UnsupportedNode("ListItem")),
            mdast::Node::Definition(_) => Err(ToMinimadError::UnsupportedNode("Definition")),
            mdast::Node::Paragraph(_) => Err(ToMinimadError::UnsupportedNode("Paragraph")),
        }
    }

    /// Emit a root node
    fn root(
        &mut self,
        mdast::Root {
            children,
            position: _,
        }: &'a mdast::Root,
    ) -> Result<(), ToMinimadError> {
        for child in children {
            self.node(child)?;
        }
        Ok(())
    }

    /// Complete the emission
    fn finish(self) -> minimad::Text<'a> {
        minimad::Text { lines: self.lines }
    }
}
