//! ## Configuration for the conversion
//!
//! Structures used to configure the conversion.

#[derive(Debug, Clone, Copy)]
/// Configuration of the conversion.
pub struct Configs {
    /// Spacing after each header
    ///
    /// The header at depth `n` will have spacing if `header_spacing[n-1]` is true.
    pub header_spacing: [bool; 6],
    /// How to style the links
    pub links_style: Style,
}
impl Configs {
    /// Return if a header need spacing after
    pub(crate) fn header_spacing(&self, depth: u8) -> bool {
        self.header_spacing
            .get((depth - 1) as usize)
            .copied()
            .unwrap_or(false) // default to no spacing. Only in invalid ASTs
    }
}
impl Default for Configs {
    fn default() -> Self {
        Self {
            header_spacing: [true, false, false, false, false, false],
            links_style: Style {
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
pub struct Style {
    /// Set if the node is bold
    pub bold: Option<bool>,
    /// Set if the node is italic
    pub italic: Option<bool>,
    /// Set if the node is strikeout
    pub strikeout: Option<bool>,
}
