use crate::dom::{AttrMap, Element, Node, Text};
use combine::{
    attempt, between,
    error::StreamError,
    many, many1, parser,
    parser::char::string,
    parser::{
        char::{char, letter, newline, space},
        choice::choice,
    },
    satisfy, sep_by, skip_many, ParseError, Parser, Stream,
};

fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
{
    (
        many1(letter()),
        skip_many(space().or(newline())),
        char('='),
        skip_many(space().or(newline())),
        between(char('"'), char('"'), many1(satisfy(|c: char| c != '"'))),
    )
        .map(|(key, _, _, _, value)| (key, value))
}

fn attributes<Input>() -> impl Parser<Input, Output = AttrMap>
where
    Input: Stream<Token = char>,
{
    (sep_by(attribute(), space().or(newline())))
        .map(|v: Vec<(String, String)>| v.into_iter().collect())
}

fn open_tag<Input>() -> impl Parser<Input, Output = (String, AttrMap)>
where
    Input: Stream<Token = char>,
{
    let open_tag_name = many1::<String, _, _>(letter());
    let open_tag_content = (
        open_tag_name,
        skip_many(space().or(newline())),
        attributes(),
    )
        .map(|(tag_name, _, attr_map)| (tag_name, attr_map));
    between(char('<'), char('>'), open_tag_content)
}

fn close_tag<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
{
    (char('<'), char('/'), many1(letter()), char('>')).map(|(_, _, tag_name, _)| tag_name)
}

fn nodes_<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
{
    (
        skip_many(space().or(newline())),
        attempt(many(
            (
                choice((attempt(element()), attempt(text()))),
                skip_many(space().or(newline())),
            )
                .map(|(node, _)| node),
        )),
    )
        .map(|(_, nodes)| nodes)
}

parser! {
    pub fn nodes[Input]()(Input) -> Vec<Box<Node>>
    where [Input: Stream<Token = char>]
    {
        nodes_()
    }
}

fn text<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
{
    many1(satisfy(|c: char| c != '<')).map(Text::new)
}

fn element<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
{
    (open_tag(), nodes(), close_tag()).and_then(
        |((open_tag_name, attributes), children, close_tag_name)| {
            if open_tag_name == close_tag_name {
                Ok(Element::new(open_tag_name, attributes, children))
            } else {
                Err(
                    <Input::Error as ParseError<char, _, _>>::StreamError::message_static_message(
                        "tag name of open tag and close tag mismatched",
                    ),
                )
            }
        },
    )
}

pub fn html<Input>() -> impl Parser<Input, Output = Vec<Box<Node>>>
where
    Input: Stream<Token = char>,
{
    (doctype(), nodes()).map(|(_, nodes)| nodes)
}

fn doctype<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
{
    string("<!DOCTYPE html>").map(|_| ())
}

#[cfg(test)]
mod test {
    use crate::{
        dom::{AttrMap, Element, Text},
        html::{attribute, attributes, close_tag, doctype, element, open_tag},
    };
    use combine::Parser;

    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            attribute().parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );
    }

    #[test]
    fn test_parse_attributes() {
        let mut expected_map = AttrMap::new();
        expected_map.insert("test".to_string(), "foobar".to_string());
        expected_map.insert("abc".to_string(), "def".to_string());
        assert_eq!(
            attributes().parse("test=\"foobar\" abc=\"def\""),
            Ok((expected_map, ""))
        );
        assert_eq!(attributes().parse(""), Ok((AttrMap::new(), "")))
    }

    #[test]
    fn test_parse_open_tag() {
        {
            assert_eq!(
                open_tag().parse("<p>aaaa"),
                Ok((("p".to_string(), AttrMap::new()), "aaaa"))
            );
        }
        {
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            assert_eq!(
                open_tag().parse("<p id=\"test\">"),
                Ok((("p".to_string(), attributes), ""))
            )
        }
        {
            let result = open_tag().parse("<p id=\"test\" class=\"sample\">");
            let mut attributes = AttrMap::new();
            attributes.insert("id".to_string(), "test".to_string());
            attributes.insert("class".to_string(), "sample".to_string());
            assert_eq!(result, Ok((("p".to_string(), attributes), "")));
        }

        {
            assert!(open_tag().parse("<p id>").is_err());
        }
    }

    #[test]
    fn test_parse_close_tag() {
        let result = close_tag().parse("</p>");
        assert_eq!(result, Ok(("p".to_string(), "")))
    }

    #[test]
    fn test_parse_element() {
        assert_eq!(
            element().parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            element().parse("<p>hello world</p>"),
            Ok((
                Element::new(
                    "p".to_string(),
                    AttrMap::new(),
                    vec![Text::new("hello world".to_string())]
                ),
                ""
            ))
        );

        assert_eq!(
            element().parse("<div><p>hello world</p></div>"),
            Ok((
                Element::new(
                    "div".to_string(),
                    AttrMap::new(),
                    vec![Element::new(
                        "p".to_string(),
                        AttrMap::new(),
                        vec![Text::new("hello world".to_string())]
                    )],
                ),
                ""
            ))
        );

        assert!(element().parse("<p>hello world</div>").is_err());
    }

    #[test]
    fn test_parse_doctype() {
        assert_eq!(
            doctype().parse("<!DOCTYPE html><div></div>"),
            Ok(((), "<div></div>"))
        );
    }
}
