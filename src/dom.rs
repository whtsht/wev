use std::collections::HashMap;

use crate::cssom::Selector;

pub type AttrMap = HashMap<String, String>;

#[derive(Debug, PartialEq)]
pub struct Node {
    pub node_type: NodeType,
    pub children: Vec<Box<Node>>,
}

impl Node {
    pub fn to_text(&self) -> Option<String> {
        match &self.node_type {
            NodeType::Element { .. } => None,
            NodeType::Text(Text { data }) => Some(data.clone()),
        }
    }
}

pub fn select<'a>(node: &'a Node, selector: &'a Selector) -> Vec<&'a Box<Node>> {
    node.children
        .iter()
        .filter(|&n| selector.matches(n))
        .chain(node.children.iter().flat_map(|n| select(n, selector)))
        .collect()
}

#[derive(Debug, PartialEq)]
pub enum NodeType {
    Element(Element),
    Text(Text),
}

#[derive(Debug, PartialEq)]
pub struct Element {
    pub tag_name: String,
    pub attributes: AttrMap,
}

impl Element {
    pub fn new(tag_name: String, attributes: AttrMap, children: Vec<Box<Node>>) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Element(Element {
                tag_name,
                attributes,
            }),
            children,
        })
    }
}

#[derive(Debug, PartialEq)]
pub struct Text {
    pub data: String,
}

impl Text {
    pub fn new(data: String) -> Box<Node> {
        Box::new(Node {
            node_type: NodeType::Text(Text { data }),
            children: vec![],
        })
    }
}
