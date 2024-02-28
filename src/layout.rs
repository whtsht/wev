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

fn text_to_object(text: &str, area: Rect) -> LayoutObject<'_> {
    let mut texts = vec![];
    let mut y = area.y;
    for d in split_string_by_width(text, area.width as usize, 0) {
        let len = UnicodeWidthStr::width(d) as u16;
        let area = Rect {
            x: area.x,
            y,
            width: len,
            height: 1,
        };
        y += 1;

        texts.push(Text { area, data: d })
    }

    let (width, height) = (texts.last().map(|t| t.area.width).unwrap_or(0), y - area.y);
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

fn children_to_object<'a>(node: &'a StyledNode<'a>, area: Rect) -> LayoutObject<'a> {
    let mut x = area.x;
    let mut y = area.y;
    let mut objects = vec![];
    for child in node.children.iter() {
        let area = Rect {
            x: area.x,
            y,
            width: area.width,
            height: area.height,
        };
        let object = node_to_object(child, area);
        x = object.area.width;
        y += object.area.height;
        objects.push(object);
    }
    let (width, height) = (x, y - area.y);
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

pub fn node_to_object<'a>(node: &'a StyledNode<'a>, area: Rect) -> LayoutObject<'a> {
    match node.node_type {
        NodeType::Text(dom::Text { data }) => text_to_object(data, area),
        NodeType::Element(_) => children_to_object(node, area),
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
            text_to_object("hello world", Rect::new(0, 0, 20, 3)),
            LayoutObject {
                area: Rect::new(0, 0, 11, 1),
                ty: LayoutObjectType::Texts(vec![Text {
                    area: Rect::new(0, 0, 11, 1),
                    data: "hello world"
                }])
            }
        );

        assert_eq!(
            text_to_object("hello world", Rect::new(0, 0, 3, 10)),
            LayoutObject {
                area: Rect::new(0, 0, 2, 4),
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
            text_to_object("hello world", Rect::new(3, 6, 5, 10)),
            LayoutObject {
                area: Rect::new(3, 6, 1, 3),
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
            children_to_object(&node, Rect::new(0, 0, 80, 40)),
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
        // let html = r#"
        // <div>
        //     aaa
        //     <strong>bbbbb</strong>
        // </div>
        //     "#;
        // let css = r#"strong { display: inline; }"#;
        // let node = &crate::html::html().parse(html).unwrap().0[0];
        // let stylesheet = crate::css::stylesheet(css);
        //
        // let node = crate::style::to_styled_node(node, &stylesheet).unwrap();
        // assert_eq!(
        //     children_to_object(&node, Rect::new(0, 0, 80, 40)),
        //     LayoutObject {
        //         area: Rect::new(0, 0, 8, 1),
        //         ty: LayoutObjectType::Block {
        //             children: vec![
        //                 LayoutObject {
        //                     area: Rect::new(0, 0, 3, 1),
        //                     ty: LayoutObjectType::Block {
        //                         children: vec![LayoutObject {
        //                             area: Rect::new(0, 0, 3, 1),
        //                             ty: LayoutObjectType::Texts(vec![Text {
        //                                 area: Rect::new(0, 0, 3, 1),
        //                                 data: "aaa"
        //                             }])
        //                         },]
        //                     }
        //                 },
        //                 LayoutObject {
        //                     area: Rect::new(3, 0, 5, 1),
        //                     ty: LayoutObjectType::Block {
        //                         children: vec![LayoutObject {
        //                             area: Rect::new(3, 0, 5, 1),
        //                             ty: LayoutObjectType::Texts(vec![Text {
        //                                 area: Rect::new(3, 0, 5, 1),
        //                                 data: "bbbbb"
        //                             }])
        //                         }]
        //                     }
        //                 }
        //             ]
        //         }
        //     }
        // );
    }
}
