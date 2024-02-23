use ratatui::prelude::Direction;

use crate::{
    cssom::CSSValue,
    dom::{NodeType, Text},
    style::StyledNode,
};

#[derive(Debug, PartialEq, Eq)]
pub struct LayoutObject<'a> {
    pub width: u16,
    pub height: u16,
    pub ty: LayoutObjectType<'a>,
}

#[derive(Debug, PartialEq, Eq)]
pub enum LayoutObjectType<'a> {
    Box {
        direction: Direction,
        border: bool,
        children: Vec<Box<LayoutObject<'a>>>,
    },
    Text(&'a str),
}

pub fn inline_node(node: &StyledNode) -> bool {
    match node.node_type {
        NodeType::Element(_) => {
            matches!(node.properties.get("display"), Some(CSSValue::Keyword(value)) if value == "inline")
        }
        NodeType::Text(_) => true,
    }
}

pub fn has_border(node: &StyledNode) -> bool {
    match node.node_type {
        NodeType::Element(_) => {
            matches!(node.properties.get("border"), Some(CSSValue::Keyword(value)) if value == "solid")
        }
        NodeType::Text(_) => false,
    }
}

pub fn gen_object<'a>(
    mut acc: Vec<Box<LayoutObject<'a>>>,
    node: &'a StyledNode<'a>,
) -> Vec<Box<LayoutObject<'a>>> {
    if inline_node(node) {
        if let Some(LayoutObject {
            ty:
                LayoutObjectType::Box {
                    direction,
                    children,
                    ..
                },
            ..
        }) = acc.last_mut().map(|v| v.as_mut())
        {
            if direction == &Direction::Horizontal {
                let new_node = Box::new(node_to_object(node));
                children.push(new_node);
                return acc;
            }
        }

        acc.push(Box::new(LayoutObject {
            width: 0,
            height: 0,
            ty: LayoutObjectType::Box {
                direction: Direction::Horizontal,
                border: has_border(node),
                children: vec![Box::new(node_to_object(node))],
            },
        }));
        return acc;
    }

    acc.push(Box::new(node_to_object(node)));
    acc
}

pub fn node_to_object<'a>(node: &'a StyledNode<'a>) -> LayoutObject<'a> {
    match node.node_type {
        NodeType::Element(_) => LayoutObject {
            width: 0,
            height: 0,
            ty: LayoutObjectType::Box {
                direction: Direction::Vertical,
                border: has_border(node),
                children: node.children.iter().fold(vec![], gen_object),
            },
        },
        NodeType::Text(Text { data }) => LayoutObject {
            width: data.len() as u16,
            height: 1,
            ty: LayoutObjectType::Text(data),
        },
    }
}
