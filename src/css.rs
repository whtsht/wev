use crate::cssom::*;
use combine::{
    error::StreamError,
    many, many1, optional,
    parser::{
        char::{char, letter, spaces, string},
        choice::choice,
    },
    sep_by, sep_end_by, ParseError, Parser, Stream,
};

fn css_value<Input>() -> impl Parser<Input, Output = CSSValue>
where
    Input: Stream<Token = char>,
{
    let keyword = many1(letter()).map(|s| CSSValue::Keyword(s));
    keyword
}

fn declaration<Input>() -> impl Parser<Input, Output = Declaration>
where
    Input: Stream<Token = char>,
{
    (
        many1(letter()).skip(spaces()),
        char(':').skip(spaces()),
        css_value(),
    )
        .map(|(k, _, v)| Declaration { name: k, value: v })
}

fn declarations<Input>() -> impl Parser<Input, Output = Vec<Declaration>>
where
    Input: Stream<Token = char>,
{
    sep_end_by(declaration().skip(spaces()), char(';').skip(spaces()))
}

fn selectors<Input>() -> impl Parser<Input, Output = Vec<Selector>>
where
    Input: Stream<Token = char>,
{
    sep_by(simple_selector().skip(spaces()), char(',').skip(spaces()))
}

fn simple_selector<Input>() -> impl Parser<Input, Output = SimpleSelector>
where
    Input: Stream<Token = char>,
{
    let universal_selector = char('*').map(|_| SimpleSelector::UniversalSelector);
    let class_selector = (char('.'), many1(letter()))
        .map(|(_, class_name)| SimpleSelector::ClassSelector { class_name });
    let type_or_attribute_selector = (
        many1(letter()).skip(spaces()),
        optional((
            char('[').skip(spaces()),
            many1(letter()),
            choice((string("="), string("~="))),
            many1(letter()),
            char(']'),
        )),
    )
        .and_then(|(tag_name, opts)| match opts {
            Some((_, attribute, op, value, _)) => {
                let op = match op {
                    "=" => AttributeSelectorOp::Eq,
                    "~=" => AttributeSelectorOp::Contain,
                    _ => {
                        return Err(<Input::Error as ParseError<char, _, _>>::StreamError::message_static_message(
                            "invalid attribute selector op",
                        ))
                    }
                };
                Ok(SimpleSelector::AttributeSelector {
                    tag_name,
                    attribute,
                    op,
                    value,
                })
            }
            None => Ok(SimpleSelector::TypeSelector { tag_name }),
        });

    choice((
        universal_selector,
        class_selector,
        type_or_attribute_selector,
    ))
}

fn rule<Input>() -> impl Parser<Input, Output = Rule>
where
    Input: Stream<Token = char>,
{
    (
        selectors().skip(spaces()),
        char('{').skip(spaces()),
        declarations().skip(spaces()),
        char('}'),
    )
        .map(|(selectors, _, declarations, _)| Rule {
            selectors,
            declarations,
        })
}

pub fn stylesheet(raw: &str) -> Stylesheet {
    rules()
        .parse(raw)
        .map(|(rules, _)| Stylesheet::new(rules))
        .unwrap()
}

fn rules<Input>() -> impl Parser<Input, Output = Vec<Rule>>
where
    Input: Stream<Token = char>,
    Input::Error: ParseError<Input::Token, Input::Range, Input::Position>,
{
    (spaces(), many(rule().skip(spaces()))).map(|(_, rules)| rules)
}

#[cfg(test)]
mod tests {
    use crate::{
        css::{declarations, rule, selectors, simple_selector},
        cssom::{AttributeSelectorOp, CSSValue, Declaration, Rule, SimpleSelector},
    };
    use combine::Parser;

    #[test]
    fn test_declarations() {
        assert_eq!(
            declarations().parse("foo: bar; piyo: piyopiyo;"),
            Ok((
                vec![
                    Declaration {
                        name: "foo".to_string(),
                        value: CSSValue::Keyword("bar".to_string())
                    },
                    Declaration {
                        name: "piyo".to_string(),
                        value: CSSValue::Keyword("piyopiyo".to_string())
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_selectors() {
        assert_eq!(
            selectors().parse("test [foo=bar], a"),
            Ok((
                vec![
                    SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    },
                    SimpleSelector::TypeSelector {
                        tag_name: "a".to_string(),
                    }
                ],
                ""
            ))
        );
    }

    #[test]
    fn test_simple_selector() {
        assert_eq!(
            simple_selector().parse("*"),
            Ok((SimpleSelector::UniversalSelector, ""))
        );

        assert_eq!(
            simple_selector().parse("test"),
            Ok((
                SimpleSelector::TypeSelector {
                    tag_name: "test".to_string(),
                },
                ""
            ))
        );

        assert_eq!(
            simple_selector().parse("test [foo=bar]"),
            Ok((
                SimpleSelector::AttributeSelector {
                    tag_name: "test".to_string(),
                    attribute: "foo".to_string(),
                    op: AttributeSelectorOp::Eq,
                    value: "bar".to_string()
                },
                ""
            ))
        );

        assert_eq!(
            simple_selector().parse(".test"),
            Ok((
                SimpleSelector::ClassSelector {
                    class_name: "test".to_string(),
                },
                ""
            ))
        );
    }

    #[test]
    fn test_rule() {
        assert_eq!(
            rule().parse("test [foo=bar] {}"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![]
                },
                ""
            ))
        );

        assert_eq!(
            rule().parse("test [foo=bar], testtest[piyo~=guoo] {}"),
            Ok((
                Rule {
                    selectors: vec![
                        SimpleSelector::AttributeSelector {
                            tag_name: "test".to_string(),
                            attribute: "foo".to_string(),
                            op: AttributeSelectorOp::Eq,
                            value: "bar".to_string()
                        },
                        SimpleSelector::AttributeSelector {
                            tag_name: "testtest".to_string(),
                            attribute: "piyo".to_string(),
                            op: AttributeSelectorOp::Contain,
                            value: "guoo".to_string()
                        }
                    ],
                    declarations: vec![]
                },
                ""
            ))
        );

        assert_eq!(
            rule().parse("test [foo=bar] { aa: bb; cc: dd; }"),
            Ok((
                Rule {
                    selectors: vec![SimpleSelector::AttributeSelector {
                        tag_name: "test".to_string(),
                        attribute: "foo".to_string(),
                        op: AttributeSelectorOp::Eq,
                        value: "bar".to_string()
                    }],
                    declarations: vec![
                        Declaration {
                            name: "aa".to_string(),
                            value: CSSValue::Keyword("bb".to_string())
                        },
                        Declaration {
                            name: "cc".to_string(),
                            value: CSSValue::Keyword("dd".to_string()),
                        }
                    ]
                },
                ""
            ))
        );
    }
}
