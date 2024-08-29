#![doc = include_str!("../README.md")]

use std::borrow::Cow;

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;

#[derive(Clone, Debug, Display, Error)]
/// Error while converting the AST into a `minimad` text
pub enum ToMinimadError<'a> {
    #[display("`{}` node is not supported", type_of(node))]
    UnsupportedNode { node: Cow<'a, mdast::Node> },
    #[display(
        "`{}` node is not supported as a child of a `{}` node",
        type_of(child),
        type_of(parent)
    )]
    UnsupportedChildNode {
        child: Cow<'a, mdast::Node>,
        parent: Cow<'a, mdast::Node>,
    },
}
impl<'a> ToMinimadError<'a> {
    fn unsupported_node(node: &'a mdast::Node) -> Self {
        Self::UnsupportedNode {
            node: Cow::Borrowed(node),
        }
    }
    fn unsupported_child_node(child: &'a mdast::Node, parent: &'a mdast::Node) -> Self {
        Self::UnsupportedChildNode {
            child: Cow::Borrowed(child),
            parent: Cow::Borrowed(parent),
        }
    }

    /// Make this error 'static, cloning the nodes it refers to
    ///
    /// This enable an error borrowing from the AST to be bubbled up over the AST itself
    pub fn into_static(self) -> ToMinimadError<'static> {
        match self {
            ToMinimadError::UnsupportedNode { node } => ToMinimadError::UnsupportedNode {
                node: Cow::Owned(node.into_owned()),
            },
            ToMinimadError::UnsupportedChildNode { child, parent } => {
                ToMinimadError::UnsupportedChildNode {
                    child: Cow::Owned(child.into_owned()),
                    parent: Cow::Owned(parent.into_owned()),
                }
            }
        }
    }
}

/// Convert the markdown AST to a minimad Text
pub fn to_minimad<'a>(ast: &'a mdast::Node) -> Result<minimad::Text<'a>, ToMinimadError<'a>> {
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
    _options: Options,
}
impl<'a> Emitter<'a> {
    /// Create a new, empty emitter
    fn new(options: Options) -> Self {
        Self {
            lines: vec![],
            _options: options,
        }
    }

    /// Emit a node
    fn node(&mut self, node: &'a mdast::Node) -> Result<(), ToMinimadError<'a>> {
        // emit the node
        match node {
            // Catch all for unsupported nodes
            other => Err(ToMinimadError::unsupported_node(other)),
        }
    }

    /// Complete the emission
    fn finish(self) -> minimad::Text<'a> {
        minimad::Text { lines: self.lines }
    }
}

/// Find a name for a node
///
/// Used for error messages
fn type_of(node: &mdast::Node) -> &'static str {
    match node {
        mdast::Node::Root(_) => "Root",
        mdast::Node::BlockQuote(_) => "BlockQuote",
        mdast::Node::FootnoteDefinition(_) => "FootnoteDefinition",
        mdast::Node::MdxJsxFlowElement(_) => "MdxJsxFlowElement",
        mdast::Node::List(_) => "List",
        mdast::Node::MdxjsEsm(_) => "MdxjsEsm",
        mdast::Node::Toml(_) => "Toml",
        mdast::Node::Yaml(_) => "Yaml",
        mdast::Node::Break(_) => "Break",
        mdast::Node::InlineCode(_) => "InlineCode",
        mdast::Node::InlineMath(_) => "InlineMath",
        mdast::Node::Delete(_) => "Delete",
        mdast::Node::Emphasis(_) => "Emphasis",
        mdast::Node::MdxTextExpression(_) => "MdxTextExpression",
        mdast::Node::FootnoteReference(_) => "FootnoteReference",
        mdast::Node::Html(_) => "Html",
        mdast::Node::Image(_) => "Image",
        mdast::Node::ImageReference(_) => "ImageReference",
        mdast::Node::MdxJsxTextElement(_) => "MdxJsxTextElement",
        mdast::Node::Link(_) => "Link",
        mdast::Node::LinkReference(_) => "LinkReference",
        mdast::Node::Strong(_) => "Strong",
        mdast::Node::Text(_) => "Text",
        mdast::Node::Code(_) => "Code",
        mdast::Node::Math(_) => "Math",
        mdast::Node::MdxFlowExpression(_) => "MdxFlowExpression",
        mdast::Node::Heading(_) => "Heading",
        mdast::Node::Table(_) => "Table",
        mdast::Node::ThematicBreak(_) => "ThematicBreak",
        mdast::Node::TableRow(_) => "TableRow",
        mdast::Node::TableCell(_) => "TableCell",
        mdast::Node::ListItem(_) => "ListItem",
        mdast::Node::Definition(_) => "Definition",
        mdast::Node::Paragraph(_) => "Paragraph",
    }
}
