use combine::Parser;
use std::io::Result;
use wev::{css, html, style::to_styled_node};

fn main() -> Result<()> {
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
            "#
        .into(),
    );
    let nodes = to_styled_node(&dom[0], &stylesheet);

    wev::start(nodes.as_ref().unwrap())
}
