use std::io::Write;

use crate::jack::tokens;

use super::parser::Element;

pub fn write_token<W: Write>(file: &mut W, t: &tokens::Token) {
    let tag = match &t.kind {
        tokens::Kind::Symbol(_) => "symbol",
        tokens::Kind::Keyword(_) => "keyword",
        tokens::Kind::Ident(_) => "identifier",
        tokens::Kind::IntConst(_) => "integerConstant",
        tokens::Kind::StrConst(_) => "stringConstant",
        tokens::Kind::Invalid(_) => "invalid",
        tokens::Kind::Comment(_) => return,
        tokens::Kind::EOF => return,
    };

    if let tokens::Kind::Symbol(lit) = t.kind {
        let symbol = match lit {
            "<" => "&lt;",
            ">" => "&gt;",
            "&" => "&amp;",
            _ => lit,
        };
        writeln!(file, "<{tag}> {} </{tag}>", symbol, tag = tag).unwrap();
    } else {
        writeln!(file, "<{tag}> {} </{tag}>", t.kind, tag = tag).unwrap();
    }
}

pub fn write_tree<W: Write>(file: &mut W, el: &Element, level: usize) {
    let indent = "  ".repeat(level);
    match el {
        Element::Node(node) => {
            writeln!(file, "{}<{}>", indent, node.kind).unwrap();
            for child in &node.children {
                write_tree(file, child, level + 1);
            }
            writeln!(file, "{}</{}>", indent, node.kind).unwrap();
        }
        Element::Token(token) => {
            write!(file, "{}", indent).unwrap();
            write_token(file, token)
        }
    }
}
