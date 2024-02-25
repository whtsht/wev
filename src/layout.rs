use ratatui::layout::Rect;
use unicode_width::UnicodeWidthStr;

use crate::{
    cssom::CSSValue,
    dom::{self, NodeType},
    style::StyledNode,
};

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutObject<'a> {
    pub area: Rect,
    pub ty: LayoutObjectType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutObjectType<'a> {
    Block { children: Vec<LayoutObject<'a>> },
    Texts(Vec<Text<'a>>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct Text<'a> {
    pub area: Rect,
    pub data: &'a str,
}

pub fn inline_node(node: &StyledNode) -> bool {
    match node.node_type {
        NodeType::Element(_) => {
            matches!(node.properties.get("display"), Some(CSSValue::Keyword(value)) if value == "inline")
        }
        NodeType::Text(_) => true,
    }
}

fn split_whitespace_keep_space(text: &str) -> Vec<&str> {
    let mut result = Vec::new();
    let mut current_index = 0;

    for (index, character) in text.char_indices() {
        if character.is_whitespace() {
            result.push(&text[current_index..=index]);
            current_index = index + 1;
        }
    }

    if current_index < text.len() {
        result.push(&text[current_index..]);
    }

    result
}

pub fn node_to_object<'a>(node: &'a StyledNode<'a>, rect: Rect) -> LayoutObject<'a> {
    match node.node_type {
        NodeType::Text(dom::Text { data }) => {
            let mut texts = vec![];
            let mut x = rect.x;
            let mut y = rect.y;
            for d in split_whitespace_keep_space(data) {
                let len = UnicodeWidthStr::width(d) as u16;
                let area = if x + (len % rect.width) > rect.width {
                    y += 1;
                    x = 0;
                    let area = Rect {
                        x,
                        y,
                        width: len,
                        height: len / rect.width + 2,
                    };
                    x = len % rect.width;
                    area
                } else {
                    let area = Rect {
                        x,
                        y,
                        width: len,
                        height: len / rect.width + 1,
                    };
                    x += len % rect.width;
                    area
                };

                texts.push(Text { area, data: d })
            }
            let width = if y > 0 { rect.width } else { x };
            let height = y;
            LayoutObject {
                area: Rect {
                    x: rect.x,
                    y: rect.y,
                    width,
                    height,
                },
                ty: LayoutObjectType::Texts(texts),
            }
        }
        NodeType::Element(_) => {
            let mut x = rect.x;
            let mut y = rect.y;
            let mut objects = vec![];
            for child in node.children.iter() {
                let area = if inline_node(child) {
                    Rect {
                        x,
                        y,
                        width: rect.width,
                        height: rect.height,
                    }
                } else {
                    y += 1;
                    Rect {
                        x: rect.x,
                        y,
                        width: rect.width,
                        height: rect.height,
                    }
                };
                let object = node_to_object(child, area);
                x = x.saturating_add(object.area.width).min(rect.width);
                if !inline_node(child) {
                    y = y.saturating_add(object.area.height).min(rect.height);
                }
                objects.push(object);
            }
            let (width, height) = (x, y);
            LayoutObject {
                area: Rect {
                    x: rect.x,
                    y: rect.y,
                    width,
                    height,
                },
                ty: LayoutObjectType::Block { children: objects },
            }
        }
    }
}
