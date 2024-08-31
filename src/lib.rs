#![doc = include_str!("../README.md")]

use std::{borrow::Cow, mem};

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;
use minimad::{Composite, Compound, Line};

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

#[derive(Debug, Clone, Copy)]
/// Options for the conversion
pub struct Options {
    /// If each header need spacing after
    pub header_spacing: [bool; 6],
    /// How to style the links
    pub links_style: Styling,
}
impl Options {
    fn header_spacing(&self, depth: u8) -> bool {
        self.header_spacing
            .get((depth - 1) as usize)
            .copied()
            .unwrap_or(false) // default to no spacing. Only in invalid ASTs
    }
}
impl Default for Options {
    fn default() -> Self {
        Self {
            header_spacing: [true, false, false, false, false, false],
            links_style: Styling {
                bold: None,
                italic: None,
                strikeout: None,
            },
        }
    }
}

/// Set up the styling of a node
///
/// If a value is none, it will follow the style of the surrounding text
#[derive(Debug, Clone, Copy, Default)]
pub struct Styling {
    /// Set if the node is bold
    pub bold: Option<bool>,
    /// Set if the node is italic
    pub italic: Option<bool>,
    /// Set if the node is strikeout
    pub strikeout: Option<bool>,
}

/// Represent the current content model of the emitter
#[derive(Debug, Clone, PartialEq, Eq)]
enum ContentModel<'a> {
    /// Flow content represent the sections of document.
    Flow {
        /// If the last flow element need spacing
        spacing: bool,
    },
    /// Phrasing content represent the text in a document, and its markup.
    Phrasing {
        /// Style of the lines
        style: minimad::CompositeStyle,
        /// Line being built
        compounds: Vec<minimad::Compound<'a>>,
    },
}
impl ContentModel<'_> {
    fn need_spacing(&self) -> bool {
        match self {
            ContentModel::Flow { spacing } => *spacing,
            ContentModel::Phrasing { .. } => {
                // Spacing has no sense between phrasing elements.
                // Should appear only in invalid ASTs
                // Defaulting to not giving it
                false
            }
        }
    }

    fn set_spacing(&mut self, new_spacing: bool) {
        match self {
            ContentModel::Flow { spacing } => *spacing = new_spacing,
            ContentModel::Phrasing { .. } => {
                // Here the spacing has no sense.
                // Should appear only in invalid ASTs
            }
        }
    }
}

/// Represent the current style of the emitter
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Style {
    bold: bool,
    italic: bool,
    strikeout: bool,
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
    options: Options,
}

// --- Emitter API ---

impl<'a> Emitter<'a> {
    /// Create a new, empty emitter
    fn new(options: Options) -> Self {
        Self {
            lines: vec![],
            model: None,
            style: Style::default(),
            options,
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
            mdast::Node::Code(code) => self.code(code),
            mdast::Node::Strong(strong) => self.strong(strong),
            mdast::Node::Emphasis(emphasis) => self.emphasis(emphasis),
            mdast::Node::InlineCode(inline_code) => self.inline_code(inline_code),
            mdast::Node::Delete(delete) => self.delete(delete),
            mdast::Node::Link(link) => self.link(link),
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
        self.phrasing(
            minimad::CompositeStyle::Header(*depth),
            self.options.header_spacing(*depth),
            |this| {
                // emit the childrens in phrasing mode
                for child in children {
                    this.node(child)?;
                }
                Ok(())
            },
        )
    }

    /// emit a `Text` node
    fn text(
        &mut self,
        mdast::Text { value, position: _ }: &'a mdast::Text,
    ) -> Result<(), ToMinimadError<'a>> {
        self.fmt_text(
            &value,
            self.style.bold,
            self.style.italic,
            false,
            self.style.strikeout,
        );
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
        self.phrasing(minimad::CompositeStyle::Paragraph, true, |this| {
            for child in children {
                this.node(child)?
            }
            Ok(())
        })
    }

    /// emit a `Code` node
    fn code(
        &mut self,
        mdast::Code {
            value,
            position: _,
            lang: _,
            meta: _,
        }: &'a mdast::Code,
    ) -> Result<(), ToMinimadError<'a>> {
        self.phrasing(minimad::CompositeStyle::Code, true, |this| {
            this.fmt_text(
                &value, false, false,
                false, // weird, but this is how minimad set is AST. Following to avoid surprises.
                false,
            );
            Ok(())
        })
    }

    /// emit a `Strong` node
    fn strong(
        &mut self,
        mdast::Strong {
            children,
            position: _,
        }: &'a mdast::Strong,
    ) -> Result<(), ToMinimadError<'a>> {
        let old_style = mem::replace(&mut self.style.bold, true);
        for child in children {
            self.node(child)?;
        }
        self.style.bold = old_style;
        Ok(())
    }

    /// emit a `Emphasis` node
    fn emphasis(
        &mut self,
        mdast::Emphasis {
            children,
            position: _,
        }: &'a mdast::Emphasis,
    ) -> Result<(), ToMinimadError<'a>> {
        let old_style = mem::replace(&mut self.style.italic, true);
        for child in children {
            self.node(child)?;
        }
        self.style.italic = old_style;
        Ok(())
    }

    /// emit a `InlineCode` node
    fn inline_code(
        &mut self,
        mdast::InlineCode { value, position: _ }: &'a mdast::InlineCode,
    ) -> Result<(), ToMinimadError<'a>> {
        self.fmt_text(
            &value,
            self.style.bold,
            self.style.italic,
            true,
            self.style.strikeout,
        );
        Ok(())
    }

    /// emit a `Delete` node
    fn delete(
        &mut self,
        mdast::Delete {
            children,
            position: _,
        }: &'a mdast::Delete,
    ) -> Result<(), ToMinimadError<'a>> {
        let old_style = mem::replace(&mut self.style.strikeout, true);
        for child in children {
            self.node(child)?;
        }
        self.style.strikeout = old_style;
        Ok(())
    }

    /// emit a `Link` node
    fn link(
        &mut self,
        mdast::Link {
            children,
            position: _,
            url: _,
            title: _,
        }: &'a mdast::Link,
    ) -> Result<(), ToMinimadError<'a>> {
        let new_style = Style {
            bold: self.options.links_style.bold.unwrap_or(self.style.bold),
            italic: self.options.links_style.italic.unwrap_or(self.style.italic),
            strikeout: self
                .options
                .links_style
                .strikeout
                .unwrap_or(self.style.strikeout),
        };
        let old_style = mem::replace(&mut self.style, new_style);
        for child in children {
            self.node(child)?;
        }
        self.style = old_style;
        Ok(())
    }
}

// -- Model switching and accessing --

impl<'a> Emitter<'a> {
    /// Temporarly change the content model to phrasing
    fn phrasing<R>(
        &mut self,
        style: minimad::CompositeStyle,
        spacing: bool,
        fun: impl FnOnce(&mut Self) -> R,
    ) -> R {
        // remove the old model, and if it was undefined set it to flow
        let mut old_model = self
            .model
            .take()
            .unwrap_or(ContentModel::Flow { spacing: false });
        if let ContentModel::Phrasing { style, compounds } = &mut old_model {
            // the old model was in the middle of a line. This can happen only in invalid ASTs, as the nodes that use `Phrasing`
            // as inner content should be called only in `Flow` model. Anyway, let's not mix up the content emitting that line
            self.lines.push(minimad::Line::Normal(Composite {
                style: *style,
                compounds: mem::take(compounds),
            }));
        }
        // put a spacing newline between flows element
        if old_model.need_spacing() {
            self.emptyline()
        }
        // set the new model as phrasing with the given style
        self.model = Some(ContentModel::Phrasing {
            style,
            compounds: vec![],
        });
        // call the inner function
        let res = fun(self);
        // put the old model back, settign spacing
        old_model.set_spacing(spacing);
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
            model @ (None | Some(ContentModel::Flow { .. })) => {
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
            Some(ContentModel::Phrasing { style, compounds }) => {
                self.lines.push(minimad::Line::Normal(Composite {
                    style: *style,
                    compounds: mem::take(compounds),
                }))
            }
            None | Some(ContentModel::Flow { .. }) => {
                // In this models a newline has no meaning. The method should only be called when in phrasing contexts.
                // Anyway ignoring to be lenient on malformed ASTs
            }
        }
    }

    /// Emit a empty line
    fn emptyline(&mut self) {
        self.lines.push(Line::new_paragraph(vec![]))
    }

    /// Emit formatted texts
    fn fmt_text(&mut self, value: &'a str, bold: bool, italic: bool, code: bool, strikeout: bool) {
        let mut lines = value.split("\r\n").flat_map(|l| l.split('\n'));
        if let Some(line) = lines.next() {
            self.line().push(Compound {
                src: line,
                bold,
                italic,
                code,
                strikeout,
            });
        }
        for line in lines {
            self.newline();
            self.line().push(Compound {
                src: line,
                bold,
                italic,
                code,
                strikeout,
            })
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
