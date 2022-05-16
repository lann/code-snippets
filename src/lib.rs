use anyhow::{Context, Result};
use spin_sdk::{
    http::{Request, Response},
    http_component,
};
use tree_sitter::{Parser, Query, QueryCursor};

#[http_component]
fn code_snippets(req: Request) -> Result<Response> {
    let contents = std::fs::read(req.uri().path())?;

    let body = match req.uri().query() {
        Some(query) => get_snippet(contents, query)?,
        None => contents,
    };

    Ok(http::Response::builder()
        .status(200)
        .header("content-type", "text/plain")
        .body(Some(body.into()))?)
}

fn get_snippet(contents: impl AsRef<[u8]>, ident: &str) -> Result<Vec<u8>> {
    let contents = contents.as_ref();
    let mut parser = Parser::new();
    parser.set_language(tree_sitter_rust::language()).unwrap();

    let tree = parser.parse(contents, None).unwrap();

    let query = format!(
        "(_ name: (identifier) @ident (#eq? @ident {:?})) @node",
        ident
    );
    let query = Query::new(tree_sitter_rust::language(), &query)?;

    let mut cursor = QueryCursor::new();
    let query_match = cursor
        .matches(&query, tree.root_node(), contents)
        .next()
        .context("no match")?;
    Ok(query_match.captures[0].node.utf8_text(contents)?.into())
}
