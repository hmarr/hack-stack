use hack_stack::jack::{debugxml, Parser, Tokenizer};

mod fixtures;

#[test]
fn test_parser_array_test() {
    let source_file = fixtures::load(&["jack", "parser", "ArrayTest", "Main.jack"]);
    let expected_xml = fixtures::load(&["jack", "parser", "ArrayTest", "Main.xml"]);
    assert_eq!(parse_tree_xml(&source_file.src), expected_xml.src);
}

#[test]
fn test_parser_square() {
    for file in &["Main", "Square", "SquareGame"] {
        let source_file = fixtures::load(&["jack", "parser", "Square", &format!("{}.jack", file)]);
        let expected_xml = fixtures::load(&["jack", "parser", "Square", &format!("{}.xml", file)]);
        assert_eq!(parse_tree_xml(&source_file.src), expected_xml.src);
    }
}

fn parse_tree_xml(src: &str) -> String {
    let tree = Parser::new(Tokenizer::new(src)).parse().unwrap();
    let mut buf = Vec::<u8>::new();
    debugxml::write_tree(&mut buf, &tree, 0);
    String::from_utf8(buf).unwrap()
}
