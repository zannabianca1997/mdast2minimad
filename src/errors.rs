//! ## Error handling
//!
//! Types used to report errors and add context to them.

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;

#[derive(Clone, Debug, Display, Error)]
/// General error happening while converting the AST into a `minimad` text
pub enum ToMinimadError {
    #[display("While emitting a `{node}` node")]
    /// Context information on a error
    WhileEmitting {
        node: &'static str,
        source: Box<ToMinimadError>,
    },
    #[display("`{node}` node is not supported")]
    /// A unsupported node type was encountered
    UnsupportedNode { node: &'static str },
    #[display("`{child}` node is not supported as a child")]
    /// A unexpected child node was encountered
    UnsupportedChildNode { child: &'static str },
    #[display("Numbered lists are not supported")]
    /// Numbered list are not supported by [`minimad`]
    UnsupportedNumberedLists,
    #[display("`minimad` supports nested list only up to 255 levels")]
    /// [`minimad`] uses [`u8`] to store list depth, supporting only 255 levels of nesting
    ListTooMuchNested,
    #[display("`minimad` does not support multiline table cells")]
    /// [`minimad`] supports only cells with a single line
    MultilineTableCell,
    #[display("A table cell can contain only normal lines")]
    /// The content of a table cell was rendered in a line type
    /// different than normal.
    ///
    /// This error should only appear when converting malformed markdown ASTs.
    InvalidLineTypeInTableCell,
}
impl ToMinimadError {
    /// Create a new error signaling a unsupported node
    pub(crate) fn unsupported_node(node: &mdast::Node) -> Self {
        Self::UnsupportedNode {
            node: type_of(node),
        }
    }

    /// Create a new error signaling a unsupported child node
    pub(crate) fn unsupported_child_node(child: &mdast::Node) -> Self {
        Self::UnsupportedChildNode {
            child: type_of(child),
        }
    }
}

/// Extension trait to provide context to the error type
pub(crate) trait WhileEmitting {
    /// Add parent information to the error
    fn while_emitting(self, parent: &mdast::Node) -> Self;
}

impl WhileEmitting for ToMinimadError {
    fn while_emitting(self, parent: &mdast::Node) -> Self {
        Self::WhileEmitting {
            node: type_of(parent),
            source: Box::new(self),
        }
    }
}

impl<T> WhileEmitting for Result<T, ToMinimadError> {
    fn while_emitting(self, parent: &mdast::Node) -> Self {
        self.map_err(|err| err.while_emitting(parent))
    }
}

/// Find a name for a node
///
/// Used for error messages
fn type_of(node: &mdast::Node) -> &'static str {
    match node {
        mdast::Node::Root(_) => "Root",
        mdast::Node::Blockquote(_) => "Blockquote",
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
