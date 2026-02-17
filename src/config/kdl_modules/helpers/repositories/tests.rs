
use super::*;
use kdl::KdlDocument;

fn first_node(input: &str) -> KdlNode {
    let doc: KdlDocument = input.parse().expect("valid kdl");
    doc.nodes().first().expect("node exists").clone()
}

#[test]
fn parse_repositories_merges_without_duplicates() {
    let mut repos = HashMap::new();
    let node1 = first_node(r#"repos:paru "core" "extra""#);
    let node2 = first_node(r#"repos:paru "extra" "multilib""#);

    parse_repositories(&node1, &mut repos).expect("parse node1");
    parse_repositories(&node2, &mut repos).expect("parse node2");

    let paru = repos.get("paru").expect("paru repos present");
    assert_eq!(
        paru,
        &vec![
            "core".to_string(),
            "extra".to_string(),
            "multilib".to_string()
        ]
    );
}

#[test]
fn parse_repositories_ignores_non_colon_syntax() {
    let mut repos = HashMap::new();
    let node = first_node(r#"repos "core""#);

    parse_repositories(&node, &mut repos).expect("parse node");

    assert!(repos.is_empty());
}
