use combine::Parser;
use std::{env, io::Result};
use wev::{css, dom::Node, html, layout::node_to_object, style::to_styled_node};

fn main() -> Result<()> {
    let args = env::args().collect::<Vec<_>>();
    let content = match args[1].as_str() {
        "-w" => wev::request::html_from_www(&args[2]).unwrap(),
        "-l" => wev::request::html_from_local(&args[2]).unwrap(),
        _ => panic!("argument `{}` is not supported", args[1]),
    };
    let node = html::html().parse(content.as_str()).unwrap().0;

    let root_node = Box::new(Node {
        node_type: wev::dom::NodeType::Element(wev::dom::Element {
            tag_name: "".into(),
            attributes: vec![].into_iter().collect(),
        }),
        children: node,
    });

    let style_tag = wev::cssom::SimpleSelector::TypeSelector {
        tag_name: "style".into(),
    };
    let css = wev::dom::select(&root_node, &style_tag);

    let css = css
        .first()
        .and_then(|n| n.children.first())
        .and_then(|style| style.to_text())
        .unwrap_or_default();

    let stylesheet = css::stylesheet(&css);
    let nodes = to_styled_node(&root_node, &stylesheet);
    let object = node_to_object(nodes.as_ref().unwrap());

    wev::start(&object)
}
