#![doc = include_str!("../README.md")]

use derive_more::derive::{Debug, Display, Error};
pub use markdown::mdast;

#[derive(Clone, Debug, Display, Error)]
pub enum ToMinimadError {
    #[display("`{_0}` node is not supported")]
    UnsupportedNode(#[error(not(source))] &'static str),
    #[display("A compound used the `position` property to put itself on an invalid line")]
    CompoundInsideInvalidLine,
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
    fn node(&mut self, node: &'a mdast::Node) -> Result<(), ToMinimadError> {
        // emit the node
        match node {
            _ => todo!(),
        }
    }

    /// Complete the emission
    fn finish(self) -> minimad::Text<'a> {
        minimad::Text { lines: self.lines }
    }
}
