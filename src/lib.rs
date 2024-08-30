#![doc = include_str!("../README.md")]

use std::{borrow::Cow, mem};

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;
use minimad::{Composite, Compound};

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

/// Represent the current content model of the emitter
#[derive(Debug, Clone, PartialEq, Eq)]
enum ContentModel<'a> {
    /// Flow content represent the sections of document.
    Flow,
    /// Phrasing content represent the text in a document, and its markup.
    Phrasing {
        /// Style of the lines
        style: minimad::CompositeStyle,
        /// Line being built
        compounds: Vec<minimad::Compound<'a>>,
    },
}

/// Represent the current style of the emitter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Style {
    pub bold: bool,
    pub italic: bool,
    pub strikeout: bool,
}
impl Default for Style {
    fn default() -> Self {
        Self {
            bold: false,
            italic: false,
            strikeout: false,
        }
    }
}

/// Minimad code emitter
struct Emitter<'a> {
    /// Lines already emitted
    lines: Vec<minimad::Line<'a>>,
    /// Current content model
    model: Option<ContentModel<'a>>,
    /// Current style of the emitter
    style: Style,
    /// Conversion options
    _options: Options,
}

// --- Emitter API ---

impl<'a> Emitter<'a> {
    /// Create a new, empty emitter
    fn new(options: Options) -> Self {
        Self {
            lines: vec![],
            model: None,
            style: Style::default(),
            _options: options,
        }
    }

    /// Complete the emission
    fn finish(self) -> minimad::Text<'a> {
        minimad::Text { lines: self.lines }
    }

    /// Emit an arbitrary node
    fn node(&mut self, node: &'a mdast::Node) -> Result<(), ToMinimadError<'a>> {
        // emit the node
        match node {
            mdast::Node::Root(root) => self.root(root),
            mdast::Node::Heading(heading) => self.heading(heading),
            mdast::Node::Text(text) => self.text(text),
            mdast::Node::Paragraph(paragraph) => self.paragraph(paragraph),
            // Catch all for unsupported nodes
            other => Err(ToMinimadError::unsupported_node(other)),
        }
    }
}

// -- Implementation of all supported node type --

impl<'a> Emitter<'a> {
    /// emit a `Root` node
    fn root(
        &mut self,
        mdast::Root {
            children,
            position: _,
        }: &'a mdast::Root,
    ) -> Result<(), ToMinimadError<'a>> {
        // root does not limit his content in any way
        for child in children {
            self.node(child)?;
        }
        Ok(())
    }

    /// emit a `Heading` node
    fn heading(
        &mut self,
        mdast::Heading {
            children,
            position: _,
            depth,
        }: &'a mdast::Heading,
    ) -> Result<(), ToMinimadError<'a>> {
        // Open a new phrasing session
        self.phrasing(minimad::CompositeStyle::Header(*depth), |this| {
            // emit the childrens in phrasing mode
            for child in children {
                this.node(child)?;
            }
            Ok(())
        })
    }

    /// emit a `Text` node
    fn text(
        &mut self,
        mdast::Text { value, position: _ }: &'a mdast::Text,
    ) -> Result<(), ToMinimadError<'a>> {
        let mut lines = value.lines();
        if let Some(line) = lines.next() {
            let compound = Compound {
                src: line,
                bold: self.style.bold,
                italic: self.style.italic,
                code: false,
                strikeout: self.style.strikeout,
            };
            self.line().push(compound);
        }
        for line in lines {
            self.newline();
            let compound = Compound {
                src: line,
                bold: self.style.bold,
                italic: self.style.italic,
                code: false,
                strikeout: self.style.strikeout,
            };
            self.line().push(compound);
        }
        Ok(())
    }

    /// emit a `Paragraph` node
    fn paragraph(
        &mut self,
        mdast::Paragraph {
            children,
            position: _,
        }: &'a mdast::Paragraph,
    ) -> Result<(), ToMinimadError<'a>> {
        self.phrasing(minimad::CompositeStyle::Paragraph, |this| {
            for child in children {
                this.node(child)?
            }
            Ok(())
        })
    }
}

// -- Model switching and accessing --

impl<'a> Emitter<'a> {
    /// Temporarly change the content model to phrasing
    fn phrasing<R>(
        &mut self,
        style: minimad::CompositeStyle,
        fun: impl FnOnce(&mut Self) -> R,
    ) -> R {
        // remove the old model, and if it was undefined set it to flow
        let mut old_model = self.model.take().unwrap_or(ContentModel::Flow);
        if let ContentModel::Phrasing { style, compounds } = &mut old_model {
            // the old model was in the middle of a line. This can happen only in invalid ASTs, as the nodes that use `Phrasing`
            // as inner content should be called only in `Flow` model. Anyway, let's not mix up the content emitting that line
            self.lines.push(minimad::Line::Normal(Composite {
                style: *style,
                compounds: mem::take(compounds),
            }));
        }
        // set the new model as phrasing with the given style
        self.model = Some(ContentModel::Phrasing {
            style,
            compounds: vec![],
        });
        // call the inner function
        let res = fun(self);
        // put the old model back
        let residuals = mem::replace(&mut self.model, Some(old_model));
        // if some compounds remains, emit them
        if let Some(ContentModel::Phrasing { style, compounds }) = residuals {
            self.lines
                .push(minimad::Line::Normal(Composite { style, compounds }));
        }
        // return the function result
        res
    }

    /// Return the current line
    fn line(&mut self) -> &mut Vec<Compound<'a>> {
        match &mut self.model {
            Some(ContentModel::Phrasing {
                style: _,
                compounds,
            }) => compounds,
            model @ (None | Some(ContentModel::Flow)) => {
                // If not phrasing (only in invalid ASTs), or if the model is undefined, assume we begin a new paragraph
                *model = Some(ContentModel::Phrasing {
                    style: minimad::CompositeStyle::Paragraph,
                    compounds: vec![],
                });
                let Some(ContentModel::Phrasing {
                    style: _,
                    compounds,
                }) = model
                else {
                    unreachable!()
                };
                compounds
            }
        }
    }

    /// Start a new line
    fn newline(&mut self) {
        match &mut self.model {
            Some(ContentModel::Phrasing { style, compounds }) => todo!(),
            None | Some(ContentModel::Flow) => {
                // In this models a newline has no meaning. The method should only be called when in phrasing contexts.
                // Anyway ignoring to be lenient on malformed ASTs
            }
        }
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
