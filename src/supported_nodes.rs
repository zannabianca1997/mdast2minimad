//! ## Implementation of all supported node type
//!
//! Here all the supported nodes are implemented with a function that emit the
//! visited node.

use std::mem;

pub use markdown::mdast;
use minimad::{Composite, CompositeStyle, Compound, Line, TableRow, TableRule, Text};

use crate::{CurrentStyle, Emitter, ToMinimadError, WhileEmitting as _};

impl<'a> Emitter<'a> {
    /// emit a `Root` node
    pub(crate) fn root(
        &mut self,
        mdast::Root {
            children,
            position: _,
        }: &'a mdast::Root,
    ) -> Result<(), ToMinimadError> {
        // root does not limit his content in any way
        for child in children {
            self.emit(child)?;
        }
        Ok(())
    }

    /// emit a `Heading` node
    pub(crate) fn heading(
        &mut self,
        mdast::Heading {
            children,
            position: _,
            depth,
        }: &'a mdast::Heading,
    ) -> Result<(), ToMinimadError> {
        // Open a new phrasing session
        self.phrasing(
            minimad::CompositeStyle::Header(*depth),
            self.options.header_spacing(*depth),
            |this| {
                // emit the childrens in phrasing mode
                for child in children {
                    this.emit(child)?;
                }
                Ok(())
            },
        )
    }

    /// emit a `Text` node
    pub(crate) fn text(
        &mut self,
        mdast::Text { value, position: _ }: &'a mdast::Text,
    ) -> Result<(), ToMinimadError> {
        self.fmt_text(
            value,
            self.style.bold,
            self.style.italic,
            false,
            self.style.strikeout,
        );
        Ok(())
    }

    /// emit a `Paragraph` node
    pub(crate) fn paragraph(
        &mut self,
        mdast::Paragraph {
            children,
            position: _,
        }: &'a mdast::Paragraph,
    ) -> Result<(), ToMinimadError> {
        self.phrasing(minimad::CompositeStyle::Paragraph, true, |this| {
            for child in children {
                this.emit(child)?
            }
            Ok(())
        })
    }

    /// emit a `Code` node
    pub(crate) fn code(
        &mut self,
        mdast::Code {
            value,
            position: _,
            lang: _,
            meta: _,
        }: &'a mdast::Code,
    ) -> Result<(), ToMinimadError> {
        self.phrasing(minimad::CompositeStyle::Code, true, |this| {
            this.fmt_text(
                value, false, false,
                false, // weird, but this is how minimad set is AST. Following to avoid surprises.
                false,
            );
            Ok(())
        })
    }

    /// emit a `Strong` node
    pub(crate) fn strong(
        &mut self,
        mdast::Strong {
            children,
            position: _,
        }: &'a mdast::Strong,
    ) -> Result<(), ToMinimadError> {
        let old_style = mem::replace(&mut self.style.bold, true);
        for child in children {
            self.emit(child)?;
        }
        self.style.bold = old_style;
        Ok(())
    }

    /// emit a `Emphasis` node
    pub(crate) fn emphasis(
        &mut self,
        mdast::Emphasis {
            children,
            position: _,
        }: &'a mdast::Emphasis,
    ) -> Result<(), ToMinimadError> {
        let old_style = mem::replace(&mut self.style.italic, true);
        for child in children {
            self.emit(child)?;
        }
        self.style.italic = old_style;
        Ok(())
    }

    /// emit a `InlineCode` node
    pub(crate) fn inline_code(
        &mut self,
        mdast::InlineCode { value, position: _ }: &'a mdast::InlineCode,
    ) -> Result<(), ToMinimadError> {
        self.fmt_text(
            value,
            self.style.bold,
            self.style.italic,
            true,
            self.style.strikeout,
        );
        Ok(())
    }

    /// emit a `Delete` node
    pub(crate) fn delete(
        &mut self,
        mdast::Delete {
            children,
            position: _,
        }: &'a mdast::Delete,
    ) -> Result<(), ToMinimadError> {
        let old_style = mem::replace(&mut self.style.strikeout, true);
        for child in children {
            self.emit(child)?;
        }
        self.style.strikeout = old_style;
        Ok(())
    }

    /// emit a `Link` node
    pub(crate) fn link(
        &mut self,
        mdast::Link {
            children,
            position: _,
            url: _,
            title: _,
        }: &'a mdast::Link,
    ) -> Result<(), ToMinimadError> {
        let new_style = CurrentStyle {
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
            self.emit(child)?;
        }
        self.style = old_style;
        Ok(())
    }

    /// emit a `List` node
    pub(crate) fn list(
        &mut self,
        mdast::List {
            children,
            position: _,
            ordered,
            start: _,
            spread: _,
        }: &'a mdast::List,
    ) -> Result<(), ToMinimadError> {
        if *ordered {
            return Err(ToMinimadError::UnsupportedNumberedLists);
        }
        self.phrasing(CompositeStyle::Paragraph, true, |this| {
            for item in children {
                let item @ mdast::Node::ListItem(mdast::ListItem {
                    children,
                    position: _,
                    spread: _,
                    checked: _,
                }) = item
                else {
                    return Err(ToMinimadError::unsupported_child_node(item));
                };
                // render the child as a text
                let mut emitter = Emitter::new(this.options);
                for child in children {
                    emitter.emit(child).while_emitting(item)?;
                }
                let mut item = emitter.finish();
                // Transform the first line in a list item if is a paragraph,
                // else leave a empty list item (minimad do not support item of different type)
                if let Some(Line::Normal(Composite {
                    style: style @ CompositeStyle::Paragraph,
                    compounds: _,
                })) = item.lines.first_mut()
                {
                    *style = CompositeStyle::ListItem(0)
                } else {
                    item.lines.insert(
                        0,
                        Line::Normal(Composite {
                            style: CompositeStyle::ListItem(0),
                            compounds: vec![],
                        }),
                    )
                }
                // For each child successive line, if its a list, indent it a bit more, else add some indentation as text
                for line in item.lines.iter_mut().skip(1) {
                    match line {
                        Line::Normal(Composite { style, compounds }) => match style {
                            CompositeStyle::ListItem(indent) => {
                                *indent = indent
                                    .checked_add(1)
                                    .ok_or(ToMinimadError::ListTooMuchNested)?
                            }

                            CompositeStyle::Paragraph
                            | CompositeStyle::Header(_)
                            | CompositeStyle::Code
                            | CompositeStyle::Quote => compounds.insert(
                                0,
                                Compound {
                                    src: "  ",
                                    bold: false,
                                    italic: false,
                                    code: false,
                                    strikeout: false,
                                },
                            ),
                        },
                        Line::HorizontalRule => (),
                        Line::TableRow(_) | Line::TableRule(_) => {
                            unimplemented!("Tables are not implemented")
                        }
                        Line::CodeFence(_) => {
                            unimplemented!("Code fences are still not implemented")
                        }
                    }
                }
                // Append all the lines from the item
                this.lines.append(&mut item.lines)
            }
            Ok(())
        })
    }

    /// emit a `Table` node
    pub(crate) fn table(
        &mut self,
        mdast::Table {
            children,
            position: _,
            align,
        }: &'a mdast::Table,
    ) -> Result<(), ToMinimadError> {
        let mut rows = children.iter().map(|child| {
            let mdast::Node::TableRow(child) = child else {
                return Err(ToMinimadError::unsupported_child_node(child));
            };
            Ok(child)
        });

        self.phrasing(CompositeStyle::Paragraph, true, |this| {
            this.table_row(rows.next().unwrap()?)?;
            this.lines.push(Line::TableRule(TableRule {
                cells: align
                    .iter()
                    .map(|align| match align {
                        mdast::AlignKind::Left => minimad::Alignment::Left,
                        mdast::AlignKind::Right => minimad::Alignment::Right,
                        mdast::AlignKind::Center => minimad::Alignment::Center,
                        mdast::AlignKind::None => minimad::Alignment::Unspecified,
                    })
                    .collect(),
            }));
            for row in rows {
                this.table_row(row?)?;
            }
            Ok(())
        })
    }

    /// Emit a `TableRow` node
    pub(crate) fn table_row(
        &mut self,
        mdast::TableRow {
            children,
            position: _,
        }: &'a mdast::TableRow,
    ) -> Result<(), ToMinimadError> {
        let cells = children.iter().map(|child| {
            let mdast::Node::TableCell(mdast::TableCell {
                children,
                position: _,
            }) = child
            else {
                return Err(ToMinimadError::unsupported_child_node(child));
            };
            // render the cell as text
            let mut emitter = Emitter::new(self.options);
            for child in children {
                emitter.emit(child).while_emitting(child)?;
            }
            let Text { mut lines } = emitter.finish();
            // fail if the cell has multiple lines
            if lines.len() > 1 {
                return Err(ToMinimadError::MultilineTableCell);
            }
            // return the single line
            let line = match lines.pop() {
                Some(Line::Normal(composite)) => composite,
                Some(_) => return Err(ToMinimadError::InvalidLineTypeInTableCell),
                None => Composite {
                    style: CompositeStyle::Paragraph,
                    compounds: vec![],
                },
            };
            Ok(line)
        });

        self.lines.push(Line::TableRow(TableRow {
            cells: cells.collect::<Result<_, _>>()?,
        }));

        Ok(())
    }

    /// Emit a `ThematicBreak` node
    pub(crate) fn thematic_break(
        &mut self,
        mdast::ThematicBreak { position: _ }: &'a mdast::ThematicBreak,
    ) -> Result<(), ToMinimadError> {
        self.phrasing(CompositeStyle::Paragraph, false, |this| {
            this.lines.push(Line::HorizontalRule);
            Ok(())
        })
    }
}
