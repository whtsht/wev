use crate::dom::{AttrMap, Element, Node, Text};
use combine::{
    attempt, between,
    error::StreamError,
    many, many1, optional, parser,
    parser::char::{self, string_cmp},
    parser::{
        char::{char, letter, newline, space},
        choice::choice,
    },
    satisfy, sep_by, skip_many, ParseError, Parser, Stream,
};

fn cstring<Input>(s: &'static str) -> impl Parser<Input, Output = &str>
where
    Input: Stream<Token = char>,
{
    string_cmp(s, |l, r| l.eq_ignore_ascii_case(&r))
}

fn ignore<Input>(p: impl Parser<Input>) -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
{
    p.map(|_| ())
}

fn is_ascii_whitespace(c: char) -> bool {
    // TAB
    c == '\u{0009}' ||
        // LF
        c == '\u{000A}'||
        // FF
        c == '\u{000C}'||
        // CR
        c == '\u{000D}' ||
        // SPACE
        c == '\u{0020}'
}

fn ascii_whitespace<Input>() -> impl Parser<Input, Output = char>
where
    Input: Stream<Token = char>,
{
    satisfy(is_ascii_whitespace)
}

fn attribute_name<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
{
    many1(satisfy(|c| {
        c != ' ' && c != '"' && c != '\'' && c != '>' && c != '/' && c != '='
    }))
}

fn unquoted_attribute_value<Input>() -> impl Parser<Input, Output = String>
where
    Input: Stream<Token = char>,
{
    many1(satisfy(|c: char| {
        !is_ascii_whitespace(c)
            && c != '"'
            && c != '\''
            && c != '='
            && c != '<'
            && c != '>'
            && c != '`'
    }))
}

fn empty_attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
{
    attribute_name().map(|key| (key, String::new()))
}

fn unquoted_attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
{
    (
        attribute_name(),
        skip_many(ascii_whitespace()),
        char('='),
        skip_many(ascii_whitespace()),
        unquoted_attribute_value(),
    )
        .map(|(key, _, _, _, value)| (key, value))
}

fn single_quoted_attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
{
    (
        attribute_name(),
        skip_many(ascii_whitespace()),
        char('='),
        skip_many(ascii_whitespace()),
        between(char('\''), char('\''), many(satisfy(|c| c != '\''))),
    )
        .map(|(key, _, _, _, value)| (key, value))
}

fn double_quoted_attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
{
    (
        attribute_name(),
        skip_many(ascii_whitespace()),
        char('='),
        skip_many(ascii_whitespace()),
        between(char('"'), char('"'), many(satisfy(|c| c != '"'))),
    )
        .map(|(key, _, _, _, value)| (key, value))
}

fn attribute<Input>() -> impl Parser<Input, Output = (String, String)>
where
    Input: Stream<Token = char>,
{
    choice((
        attempt(single_quoted_attribute()),
        attempt(double_quoted_attribute()),
        attempt(unquoted_attribute()),
        attempt(empty_attribute()),
    ))
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
                choice((
                    attempt(normal_element()),
                    attempt(void_element()),
                    attempt(text()),
                )),
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

fn void_element<Input>() -> impl Parser<Input, Output = Box<Node>>
where
    Input: Stream<Token = char>,
{
    open_tag().map(|(tag_name, attributes)| Element::new(tag_name, attributes, vec![]))
}

fn normal_element<Input>() -> impl Parser<Input, Output = Box<Node>>
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
    (optional(attempt(doctype())), nodes()).map(|(_, nodes)| nodes)
}

fn doctype<Input>() -> impl Parser<Input, Output = ()>
where
    Input: Stream<Token = char>,
{
    ignore((
        cstring("<!DOCTYPE"),
        attempt(many::<(), _, _>(ignore(satisfy(|c| c != '>')))),
        char('>'),
    ))
}

#[cfg(test)]
mod test {
    use crate::{
        dom::{AttrMap, Element, Text},
        html::{attribute, attributes, close_tag, doctype, normal_element, open_tag, void_element},
    };
    use combine::Parser;

    #[test]
    fn test_parse_attribute() {
        assert_eq!(
            attribute().parse("test=\"foobar\""),
            Ok((("test".to_string(), "foobar".to_string()), ""))
        );

        assert_eq!(
            attribute().parse("http-equiv='foobar'"),
            Ok((("http-equiv".to_string(), "foobar".to_string()), ""))
        );

        assert_eq!(
            attribute().parse("value=yes"),
            Ok((("value".to_string(), "yes".to_string()), ""))
        );

        assert_eq!(
            attribute().parse("disabled"),
            Ok((("disabled".to_string(), "".to_string()), ""))
        );

        assert_eq!(
            attribute().parse(r#"content="text/html; charset=utf8""#),
            Ok((
                ("content".to_string(), "text/html; charset=utf8".to_string()),
                ""
            ))
        )
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
            let mut attributes = AttrMap::new();
            attributes.insert("disabled".to_string(), "".to_string());
            assert_eq!(
                open_tag().parse("<input disabled>"),
                Ok((("input".to_string(), attributes), ""))
            );
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
            normal_element().parse("<p></p>"),
            Ok((Element::new("p".to_string(), AttrMap::new(), vec![]), ""))
        );

        assert_eq!(
            normal_element().parse("<p>hello world</p>"),
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
            normal_element().parse("<div><p>hello world</p></div>"),
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

        assert!(normal_element().parse("<p>hello world</div>").is_err());
    }

    #[test]
    fn test_parse_doctype() {
        assert_eq!(
            doctype().parse("<!DOCTYPE html><div></div>"),
            Ok(((), "<div></div>"))
        );
        assert_eq!(
            doctype().parse(r#"<!DOCTYPE HTML PUBLIC "-//W3C//DTD HTML 4.0 Transitional//EN">"#),
            Ok(((), ""))
        )
    }

    #[test]
    fn test_void_element() {
        assert_eq!(
            void_element().parse(r#"<br>"#),
            Ok((Element::new("br".to_string(), AttrMap::new(), vec![]), ""))
        );
        let mut attributes = AttrMap::new();
        attributes.insert("content".to_string(), "text/html; charset=utf8".to_string());
        attributes.insert("http-equiv".to_string(), "Content-Type".to_string());

        assert_eq!(
            void_element()
                .parse(r#"<META content="text/html; charset=utf8" http-equiv=Content-Type>"#),
            Ok((Element::new("META".to_string(), attributes, vec![]), ""))
        );
    }
}
