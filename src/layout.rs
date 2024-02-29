use ratatui::layout::Rect;
use unicode_segmentation::UnicodeSegmentation;
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

#[derive(Debug, PartialEq, Eq)]
pub struct BlockObject<'a> {
    pub area: Rect,
    pub children: Vec<BlockObjectChild<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum BlockObjectChild<'a> {
    BlockObject(BlockObject<'a>),
    InlineObject(InlineObject<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct InlineObject<'a> {
    pub area: Rect,
    pub children: Vec<InlineObjectChild<'a>>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum InlineObjectChild<'a> {
    InlineObject(InlineObject<'a>),
    TextObject(TextObject<'a>),
}

#[derive(Debug, PartialEq, Eq)]
pub struct TextObject<'a> {
    pub area: Rect,
    pub content: Vec<&'a str>,
}

pub fn inline_node(node: &StyledNode) -> bool {
    match node.node_type {
        NodeType::Element(_) => {
            matches!(node.properties.get("display"), Some(CSSValue::Keyword(value)) if value == "inline")
        }
        NodeType::Text(_) => true,
    }
}

fn split_string_by_width(text: &str, width: usize, offset: usize) -> Vec<&str> {
    let mut result = Vec::new();
    let mut curr_width = offset;
    let mut prev_index = 0;
    let mut curr_index = 0;

    for grapheme in text.graphemes(true) {
        if curr_width + grapheme.width() > width {
            result.push(&text[prev_index..curr_index]);
            prev_index = curr_index;
            curr_width = grapheme.width();
        } else {
            curr_width += grapheme.width();
        }
        curr_index += grapheme.len();
    }

    result.push(&text[prev_index..]);

    result
}

fn text_object(text: &str, x: u16, y: u16) -> TextObject<'_> {
    TextObject {
        area: Rect {
            x,
            y,
            width: text.width() as u16,
            height: 1,
        },
        content: vec![text],
    }
}

fn inline_object<'a>(node: &'a StyledNode<'a>, x: u16, y: u16) -> InlineObject<'a> {
    let width = 0;
    InlineObject {
        area: Rect {
            x,
            y,
            width,
            height: 1,
        },
        children: vec![],
    }
}

fn text_to_object(text: &str, area: Rect, offset: usize) -> LayoutObject<'_> {
    let mut texts = vec![];
    let mut y = area.y;
    let mut content_len = 0;
    for d in split_string_by_width(text, area.width as usize, offset) {
        let len = UnicodeWidthStr::width(d) as u16;
        let area = Rect {
            x: area.x,
            y,
            width: len,
            height: 1,
        };
        y += 1;
        content_len += len;

        texts.push(Text { area, data: d })
    }

    let (width, height) = (content_len, 1);
    LayoutObject {
        area: Rect {
            x: area.x,
            y: area.y,
            width,
            height,
        },
        ty: LayoutObjectType::Texts(texts),
    }
}

fn children_to_object<'a>(node: &'a StyledNode<'a>, area: Rect, offset: usize) -> LayoutObject<'a> {
    let mut y = area.y;
    let mut height = 0;
    let mut objects = vec![];
    let mut content_len = offset as u16;
    let mut width = 0;
    for child in node.children.iter() {
        let area = Rect {
            x: area.x + (content_len % area.width),
            y,
            width: area.width,
            height: area.height,
        };
        let object = node_to_object(child, area, offset);
        content_len += object.area.width;
        if !inline_node(child) {
            y += object.area.height;
            height += object.area.height;
            if width < content_len {
                width = content_len;
            }
            content_len = 0;
        } else {
            y = area.y + content_len / area.width;
            height = (content_len + area.width - 1) / area.width;
        }
        objects.push(object);
    }
    if width < content_len {
        width = content_len;
    }

    LayoutObject {
        area: Rect {
            x: area.x,
            y: area.y,
            width,
            height,
        },
        ty: LayoutObjectType::Block { children: objects },
    }
}

pub fn node_to_object<'a>(node: &'a StyledNode<'a>, area: Rect, offset: usize) -> LayoutObject<'a> {
    match node.node_type {
        NodeType::Text(dom::Text { data }) => text_to_object(data, area, offset),
        NodeType::Element(_) => children_to_object(node, area, offset),
    }
}

#[cfg(test)]
mod tests {
    use super::split_string_by_width;
    use crate::layout::{children_to_object, text_to_object, LayoutObject, LayoutObjectType, Text};
    use combine::Parser;
    use ratatui::layout::Rect;

    #[test]
    fn test_split_string_by_width() {
        assert_eq!(
            split_string_by_width("hello world", 3, 0),
            vec!["hel", "lo ", "wor", "ld"]
        );

        assert_eq!(
            split_string_by_width("こんにちは、今日はいい天気ですね。", 4, 0),
            vec![
                "こん", "にち", "は、", "今日", "はい", "い天", "気で", "すね", "。"
            ]
        );

        assert_eq!(
            split_string_by_width("こんにちは、今日はいい天気ですね。", 4, 0),
            split_string_by_width("こんにちは、今日はいい天気ですね。", 5, 0),
        );

        assert_eq!(
            split_string_by_width("こんにちは、今日はいい天気ですね。", 6, 2),
            vec!["こん", "にちは", "、今日", "はいい", "天気で", "すね。"]
        );
    }

    #[test]
    fn test_text_to_object() {
        assert_eq!(
            text_to_object("hello world", Rect::new(0, 0, 20, 3), 0),
            LayoutObject {
                area: Rect::new(0, 0, 11, 1),
                ty: LayoutObjectType::Texts(vec![Text {
                    area: Rect::new(0, 0, 11, 1),
                    data: "hello world"
                }])
            }
        );

        assert_eq!(
            text_to_object("hello world", Rect::new(0, 0, 3, 10), 0),
            LayoutObject {
                area: Rect::new(0, 0, 11, 1),
                ty: LayoutObjectType::Texts(vec![
                    Text {
                        area: Rect::new(0, 0, 3, 1),
                        data: "hel"
                    },
                    Text {
                        area: Rect::new(0, 1, 3, 1),
                        data: "lo "
                    },
                    Text {
                        area: Rect::new(0, 2, 3, 1),
                        data: "wor"
                    },
                    Text {
                        area: Rect::new(0, 3, 2, 1),
                        data: "ld"
                    }
                ])
            }
        );

        assert_eq!(
            text_to_object("hello world", Rect::new(3, 6, 5, 10), 0),
            LayoutObject {
                area: Rect::new(3, 6, 11, 1),
                ty: LayoutObjectType::Texts(vec![
                    Text {
                        area: Rect::new(3, 6, 5, 1),
                        data: "hello"
                    },
                    Text {
                        area: Rect::new(3, 7, 5, 1),
                        data: " worl"
                    },
                    Text {
                        area: Rect::new(3, 8, 1, 1),
                        data: "d"
                    },
                ])
            }
        );

        assert_eq!(
            text_to_object("hello world", Rect::new(3, 6, 5, 10), 4),
            LayoutObject {
                area: Rect::new(3, 6, 11, 1),
                ty: LayoutObjectType::Texts(vec![
                    Text {
                        area: Rect::new(3, 6, 1, 1),
                        data: "h"
                    },
                    Text {
                        area: Rect::new(3, 7, 5, 1),
                        data: "ello "
                    },
                    Text {
                        area: Rect::new(3, 8, 5, 1),
                        data: "world"
                    },
                ])
            }
        );
    }

    #[test]
    fn test_children_to_object() {
        let html = r#"
        <div>
            <div>aaa</div>
            <div>bbbbb</div>
        </div>
            "#;
        let css = r#""#;
        let node = &crate::html::html().parse(html).unwrap().0[0];
        let stylesheet = crate::css::stylesheet(css);

        let node = crate::style::to_styled_node(node, &stylesheet).unwrap();
        assert_eq!(
            children_to_object(&node, Rect::new(0, 0, 80, 40), 0),
            LayoutObject {
                area: Rect::new(0, 0, 5, 2),
                ty: LayoutObjectType::Block {
                    children: vec![
                        LayoutObject {
                            area: Rect::new(0, 0, 3, 1),
                            ty: LayoutObjectType::Block {
                                children: vec![LayoutObject {
                                    area: Rect::new(0, 0, 3, 1),
                                    ty: LayoutObjectType::Texts(vec![Text {
                                        area: Rect::new(0, 0, 3, 1),
                                        data: "aaa"
                                    }])
                                },]
                            }
                        },
                        LayoutObject {
                            area: Rect::new(0, 1, 5, 1),
                            ty: LayoutObjectType::Block {
                                children: vec![LayoutObject {
                                    area: Rect::new(0, 1, 5, 1),
                                    ty: LayoutObjectType::Texts(vec![Text {
                                        area: Rect::new(0, 1, 5, 1),
                                        data: "bbbbb"
                                    }])
                                }]
                            }
                        }
                    ]
                }
            }
        );
        let html = r#"
            <div>とても<strong>強い</strong></div>
                "#;
        let css = r#"strong { display: inline; }"#;
        let node = &crate::html::html().parse(html).unwrap().0[0];
        let stylesheet = crate::css::stylesheet(css);

        let node = crate::style::to_styled_node(node, &stylesheet).unwrap();
        assert_eq!(
            children_to_object(&node, Rect::new(0, 0, 80, 40), 0),
            LayoutObject {
                area: Rect::new(0, 0, 10, 1),
                ty: LayoutObjectType::Block {
                    children: vec![
                        LayoutObject {
                            area: Rect::new(0, 0, 6, 1),
                            ty: LayoutObjectType::Texts(vec![Text {
                                area: Rect::new(0, 0, 6, 1),
                                data: "とても"
                            }])
                        },
                        LayoutObject {
                            area: Rect::new(6, 0, 4, 1),
                            ty: LayoutObjectType::Block {
                                children: vec![LayoutObject {
                                    area: Rect::new(6, 0, 4, 1),
                                    ty: LayoutObjectType::Texts(vec![Text {
                                        area: Rect::new(6, 0, 4, 1),
                                        data: "強い"
                                    }])
                                }]
                            }
                        }
                    ]
                }
            }
        );
    }
}
