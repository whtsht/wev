use combine::Parser;
use std::io::Result;
use wev::{css, dom::Node, html, style::to_styled_node};

fn main() -> Result<()> {
    let node = html::nodes()
        .parse(
            r#"
<body>
  <div class="none"><p>this should not be shown</p></div>
  <style>
    .none {
      display: none;
    }
    .inline {
      display: inline;
    }
  </style>
</body>"#,
        )
        .unwrap()
        .0;

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
        .get(0)
        .and_then(|n| n.children.get(0))
        .and_then(|style| style.to_text())
        .unwrap_or_default();

    let stylesheet = css::stylesheet(&css);
    let nodes = to_styled_node(&root_node, &stylesheet);

    wev::start(nodes.as_ref().unwrap())
}
