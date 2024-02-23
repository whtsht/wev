use crate::{
    cssom::{CSSValue, Stylesheet},
    dom::{Node, NodeType},
};
use std::collections::HashMap;

/// `StyledNode` wraps `Node` with related CSS properties.
/// It forms a tree as `Node` does.
#[derive(Debug, PartialEq)]
pub struct StyledNode<'a> {
    pub node_type: &'a NodeType,
    pub children: Vec<StyledNode<'a>>,

    pub properties: HashMap<String, CSSValue>,
}

pub fn to_styled_node<'a>(node: &'a Box<Node>, stylesheet: &Stylesheet) -> Option<StyledNode<'a>> {
    let mut properties: HashMap<String, (u32, CSSValue)> = HashMap::new();

    for matched_rule in stylesheet.rules.iter().filter(|r| r.matches(node)) {
        for (selector, declaration) in matched_rule
            .selectors
            .iter()
            .zip(matched_rule.declarations.iter())
        {
            if let Some((specificity, _)) = properties.get(&declaration.name) {
                if *specificity <= selector.specificity() {
                    properties.insert(
                        declaration.name.clone(),
                        (selector.specificity(), declaration.value.clone()),
                    );
                }
            } else {
                properties.insert(
                    declaration.name.clone(),
                    (selector.specificity(), declaration.value.clone()),
                );
            }
        }
    }

    if properties.get("display").is_none() {
        match node.node_type {
            NodeType::Element(ref element) => match element.tag_name.as_str() {
                "area" | "base" | "basefont" | "datalist" | "head" | "link" | "meta"
                | "noembed" | "noframes" | "param" | "rp" | "script" | "style" | "template"
                | "title" => {
                    properties.insert("display".into(), (0, CSSValue::Keyword("none".into())));
                }
                _ => {
                    properties.insert("display".into(), (0, CSSValue::Keyword("block".into())));
                }
            },
            NodeType::Text(_) => {}
        }
    }

    if properties.get("font-weight").is_none() {
        match node.node_type {
            NodeType::Element(ref element) => match element.tag_name.as_str() {
                "b" | "strong" => {
                    properties.insert("font-weight".into(), (0, CSSValue::Keyword("bold".into())));
                }
                _ => {
                    properties.insert(
                        "font-weight".into(),
                        (0, CSSValue::Keyword("normal".into())),
                    );
                }
            },
            NodeType::Text(_) => {}
        }
    }

    if properties.get("display").map(|v| &v.1) == Some(&CSSValue::Keyword("none".into())) {
        return None;
    }

    let children = node
        .children
        .iter()
        .filter_map(|x| to_styled_node(x, stylesheet))
        .collect();

    let properties = properties.into_iter().map(|(k, v)| (k, v.1)).collect();
    Some(StyledNode {
        node_type: &node.node_type,
        properties,
        children,
    })
}

#[cfg(test)]
mod tests {
    use combine::Parser;

    use crate::{
        css,
        cssom::CSSValue,
        dom::{Element, NodeType, Text},
        html,
        style::StyledNode,
    };

    use super::to_styled_node;

    #[test]
    fn test_styled_node() {
        let dom = html::nodes()
            .parse("<p class=\"foo\">hello world</p>")
            .unwrap()
            .0;
        let stylesheet = css::stylesheet("p { color:red; }");
        let nodes = to_styled_node(&dom[0], &stylesheet);
        assert_eq!(
            nodes,
            Some(StyledNode {
                node_type: &NodeType::Element(Element {
                    tag_name: "p".into(),
                    attributes: vec![("class".into(), "foo".into())].into_iter().collect(),
                }),
                children: vec![StyledNode {
                    node_type: &NodeType::Text(Text {
                        data: "hello world".into()
                    }),
                    children: vec![],
                    properties: vec![].into_iter().collect()
                }],
                properties: vec![
                    ("color".into(), CSSValue::Keyword("red".into())),
                    ("font-weight".into(), CSSValue::Keyword("normal".into())),
                    ("display".into(), CSSValue::Keyword("block".into()))
                ]
                .into_iter()
                .collect()
            })
        );
    }

    #[test]
    fn test_specificity() {
        let dom = html::nodes()
            .parse(
                r#"
                <div>
                    <p foo="bar">hello world</p>
                </div>
                   "#,
            )
            .unwrap()
            .0;
        let stylesheet = css::stylesheet(
            r#"
            div {
                color:red;
            }
            p {
                color:blue;
            }
            p[foo=bar] {
                color:yellow;
            }
            "#,
        );
        let nodes = to_styled_node(&dom[0], &stylesheet);

        assert_eq!(
            nodes,
            Some(StyledNode {
                node_type: &NodeType::Element(Element {
                    tag_name: "div".into(),
                    attributes: vec![].into_iter().collect()
                }),
                children: vec![StyledNode {
                    node_type: &NodeType::Element(Element {
                        tag_name: "p".into(),
                        attributes: vec![("foo".into(), "bar".into())].into_iter().collect()
                    }),
                    children: vec![StyledNode {
                        node_type: &NodeType::Text(Text {
                            data: "hello world".into()
                        }),
                        children: vec![],
                        properties: vec![].into_iter().collect()
                    }],
                    properties: vec![
                        ("color".into(), CSSValue::Keyword("yellow".into())),
                        ("display".into(), CSSValue::Keyword("block".into())),
                        ("font-weight".into(), CSSValue::Keyword("normal".into())),
                    ]
                    .into_iter()
                    .collect()
                }],
                properties: vec![
                    ("color".into(), CSSValue::Keyword("red".into())),
                    ("display".into(), CSSValue::Keyword("block".into())),
                    ("font-weight".into(), CSSValue::Keyword("normal".into())),
                ]
                .into_iter()
                .collect()
            })
        );
    }
}
