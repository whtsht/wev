use crate::dom::{Node, NodeType};

/// `Stylesheet` represents a single stylesheet.
/// It consists of multiple rules, which are called "rule-list" in the standard (https://www.w3.org/TR/css-syntax-3/).
#[derive(Debug, PartialEq)]
pub struct Stylesheet {
    pub rules: Vec<Rule>,
}

impl Stylesheet {
    pub fn new(rules: Vec<Rule>) -> Self {
        Self { rules }
    }
}

/// `Rule` represents a single CSS rule.
#[derive(Debug, PartialEq)]
pub struct Rule {
    pub selectors: Vec<Selector>, // a comma-separated list of selectors
    pub declarations: Vec<Declaration>,
}

impl Rule {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        self.selectors.iter().any(|s| s.matches(n))
    }
}

/// NOTE: This is not compliant to the standard for simplicity.
///
/// In the standard, *a selector* is *a chain* of one or more sequences of simple selectors separated by combinators,
/// where a sequence of simple selectors is a chain of simple selectors that are not separated by a combinator.
pub type Selector = SimpleSelector;

/// `SimpleSelector` represents a simple selector defined in the following standard:
/// https://www.w3.org/TR/selectors-3/#selector-syntax
#[derive(Debug, PartialEq)]
pub enum SimpleSelector {
    UniversalSelector,
    TypeSelector {
        tag_name: String,
    },
    AttributeSelector {
        tag_name: String,
        op: AttributeSelectorOp,
        attribute: String,
        value: String,
    },
    ClassSelector {
        class_name: String,
    },
    // TODO (enhancement): support multiple attribute selectors like `a[href=bar][ping=foo]`
    // TODO (enhancement): support more attribute selectors
}

impl SimpleSelector {
    pub fn matches(&self, n: &Box<Node>) -> bool {
        match self {
            SimpleSelector::UniversalSelector => true,
            SimpleSelector::TypeSelector { tag_name } => match n.node_type {
                NodeType::Element(ref e) => e.tag_name.as_str() == tag_name,
                _ => false,
            },
            SimpleSelector::AttributeSelector {
                tag_name,
                op,
                attribute,
                value,
            } => match n.node_type {
                NodeType::Element(ref e) => {
                    e.tag_name.as_str() == tag_name
                        && match op {
                            AttributeSelectorOp::Eq => e.attributes.get(attribute) == Some(value),
                            AttributeSelectorOp::Contain => e
                                .attributes
                                .get(attribute)
                                .map(|value_| {
                                    value_
                                        .split_ascii_whitespace()
                                        .find(|v| v == value)
                                        .is_some()
                                })
                                .unwrap_or(false),
                        }
                }
                _ => false,
            },
            SimpleSelector::ClassSelector { class_name } => match n.node_type {
                NodeType::Element(ref e) => e.attributes.get("class") == Some(class_name),
                _ => false,
            },
        }
    }

    pub fn specificity(&self) -> u32 {
        match self {
            SimpleSelector::UniversalSelector => 0,
            SimpleSelector::TypeSelector { .. } => 1,
            SimpleSelector::AttributeSelector { .. } | SimpleSelector::ClassSelector { .. } => 10,
        }
    }
}

/// `AttributeSelectorOp` is an operator which is allowed to use.
/// See https://www.w3.org/TR/selectors-3/#attribute-selectors to check the full list of available operators.
#[derive(Debug, PartialEq)]
pub enum AttributeSelectorOp {
    Eq,      // =
    Contain, // ~=
}

/// `Declaration` represents a CSS declaration defined at [CSS Syntax Module Level 3](https://www.w3.org/TR/css-syntax-3/#declaration)
///
/// Declarations are further categorized into the followings:
/// - descriptors, which are mostly used in "at-rules" like `@foo (bar: piyo)` https://www.w3.org/Style/CSS/all-descriptors.en.html
/// - properties, which are mostly used in "qualified rules" like `.foo {bar: piyo}` https://www.w3.org/Style/CSS/all-descriptors.en.html
///
/// For simplicity, we handle two types of declarations together.
#[derive(Debug, PartialEq)]
pub struct Declaration {
    pub name: String,
    pub value: CSSValue,
    // TODO (enhancement): add a field for `!important`
}

/// `CSSValue` represents some of *component value types* defined at [CSS Values and Units Module Level 3](https://www.w3.org/TR/css-values-3/#component-types).
#[derive(Debug, PartialEq, Clone)]
pub enum CSSValue {
    Keyword(String),
}

#[cfg(test)]
mod tests {
    use crate::{
        cssom::{AttributeSelectorOp, SimpleSelector},
        dom::Element,
    };

    #[test]
    fn test_universal_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .into_iter()
            .collect(),
            vec![],
        );
        assert_eq!(SimpleSelector::UniversalSelector.matches(e), true);
    }

    #[test]
    fn test_type_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .into_iter()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::TypeSelector {
                tag_name: "p".into(),
            })
            .matches(e),
            true
        );

        assert_eq!(
            (SimpleSelector::TypeSelector {
                tag_name: "invalid".into(),
            })
            .matches(e),
            false
        );
    }

    #[test]
    fn test_attribute_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test test2".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .into_iter()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "test test2".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            true
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "invalid".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "invalid".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "invalid".into(),
                attribute: "id".into(),
                value: "test".into(),
                op: AttributeSelectorOp::Eq,
            })
            .matches(e),
            false
        );

        assert_eq!(
            (SimpleSelector::AttributeSelector {
                tag_name: "p".into(),
                attribute: "id".into(),
                value: "test2".into(),
                op: AttributeSelectorOp::Contain,
            })
            .matches(e),
            true
        );
    }

    #[test]
    fn test_class_selector_behaviour() {
        let e = &Element::new(
            "p".to_string(),
            [
                ("id".to_string(), "test".to_string()),
                ("class".to_string(), "testclass".to_string()),
            ]
            .into_iter()
            .collect(),
            vec![],
        );

        assert_eq!(
            (SimpleSelector::ClassSelector {
                class_name: "testclass".into(),
            })
            .matches(e),
            true
        );

        assert_eq!(
            (SimpleSelector::ClassSelector {
                class_name: "invalid".into(),
            })
            .matches(e),
            false
        );
    }
}
