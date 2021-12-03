use std::fmt::{self, Debug};

use super::tokenizer::Tokenizer;
use super::tokens::{Kind, Token};
use crate::common::{Span, SpanError};

#[derive(Debug, PartialEq)]
pub enum Element<'a> {
    Node(Node<'a>),
    Token(Token<'a>),
}

impl<'a> From<Token<'a>> for Element<'a> {
    fn from(token: Token<'a>) -> Self {
        Element::Token(token)
    }
}

impl<'a> From<Node<'a>> for Element<'a> {
    fn from(node: Node<'a>) -> Self {
        Element::Node(node)
    }
}

#[derive(Debug, PartialEq)]
pub enum NodeKind {
    Class,
    ClassVarDec,
    SubroutineDec,
    ParameterList,
    SubroutineBody,
    VarDec,
    Statements,
    LetStatement,
    IfStatement,
    WhileStatement,
    DoStatement,
    ReturnStatement,
    Expression,
    Term,
    ExpressionList,
}

impl fmt::Display for NodeKind {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            NodeKind::Class => write!(f, "class"),
            NodeKind::ClassVarDec => write!(f, "classVarDec"),
            NodeKind::SubroutineDec => write!(f, "subroutineDec"),
            NodeKind::ParameterList => write!(f, "parameterList"),
            NodeKind::SubroutineBody => write!(f, "subroutineBody"),
            NodeKind::VarDec => write!(f, "varDec"),
            NodeKind::Statements => write!(f, "statements"),
            NodeKind::LetStatement => write!(f, "letStatement"),
            NodeKind::Expression => write!(f, "expression"),
            NodeKind::Term => write!(f, "term"),
            NodeKind::ExpressionList => write!(f, "expressionList"),
            NodeKind::IfStatement => write!(f, "ifStatement"),
            NodeKind::WhileStatement => write!(f, "whileStatement"),
            NodeKind::DoStatement => write!(f, "doStatement"),
            NodeKind::ReturnStatement => write!(f, "returnStatement"),
        }
    }
}

#[derive(Debug, PartialEq)]
pub struct Node<'a> {
    pub kind: NodeKind,
    pub children: Vec<Element<'a>>,
}

impl<'a> Node<'a> {
    pub fn new(kind: NodeKind) -> Node<'a> {
        Node {
            kind,
            children: vec![],
        }
    }

    pub fn add_token(&mut self, token: Token<'a>) {
        self.children.push(token.into());
    }

    pub fn add_node(&mut self, node: Node<'a>) {
        self.children.push(node.into());
    }
}

type ParseResult<T> = Result<T, SpanError>;
type NodeResult<'a> = ParseResult<Node<'a>>;

pub struct Parser<'a> {
    tokenizer: Tokenizer<'a>,
    token: Token<'a>,
    prev_token: Token<'a>,
    peeked_token: Option<Token<'a>>,
}

impl<'a> Parser<'a> {
    pub fn new(mut tokenizer: Tokenizer<'a>) -> Parser<'a> {
        let mut token = tokenizer.next_token();
        while matches!(token.kind, Kind::Comment(_)) {
            token = tokenizer.next_token();
        }
        Parser {
            tokenizer,
            token,
            prev_token: Token::invalid("", 0),
            peeked_token: None,
        }
    }

    pub fn parse(&mut self) -> Result<Element<'a>, SpanError> {
        self.parse_class().and_then(|el| {
            if self.token.kind == Kind::EOF {
                Ok(el)
            } else {
                Err(self.unexpected_token_error("end of file"))
            }
        })
    }

    fn parse_class(&mut self) -> Result<Element<'a>, SpanError> {
        let mut node = Node::new(NodeKind::Class);
        node.add_token(self.expect_keyword(&["class"])?);
        node.add_token(self.expect_ident()?);
        node.add_token(self.expect_symbol("{")?);

        loop {
            match self.token.kind {
                Kind::Keyword(t @ ("field" | "static")) => node.add_node(self.parse_var_dec(t)?),
                Kind::Keyword("function" | "method" | "constructor") => {
                    node.add_node(self.parse_subroutine_dec()?)
                }
                _ => break,
            }
        }

        node.add_token(self.expect_symbol("}")?);

        Ok(Element::Node(node))
    }

    fn parse_var_dec(&mut self, var_type: &str) -> NodeResult<'a> {
        let mut node = Node::new(match var_type {
            "var" => NodeKind::VarDec,
            "field" | "static" => NodeKind::ClassVarDec,
            _ => unreachable!(),
        });
        node.add_token(self.expect_keyword(&[var_type])?);
        node.add_token(self.expect_type_name()?);
        node.add_token(self.expect_ident()?);
        while matches!(self.token.kind, Kind::Symbol(",")) {
            node.add_token(self.token);
            self.advance();
            node.add_token(self.expect_ident()?);
        }
        node.add_token(self.expect_symbol(";")?);

        Ok(node)
    }

    fn parse_subroutine_dec(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::SubroutineDec);
        node.add_token(self.expect_keyword(&["function", "method", "constructor"])?);
        node.add_token(self.expect_type_name()?);
        node.add_token(self.expect_ident()?);
        node.add_token(self.expect_symbol("(")?);
        node.add_node(self.parse_parameter_list()?);
        node.add_token(self.expect_symbol(")")?);

        node.add_node(self.parse_subroutine_body()?);

        Ok(node)
    }

    fn parse_parameter_list(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::ParameterList);

        while !matches!(self.token.kind, Kind::Symbol(")")) {
            node.add_token(self.expect_type_name()?);
            node.add_token(self.expect_ident()?);
            if matches!(self.token.kind, Kind::Symbol(",")) {
                node.add_token(self.token);
                self.advance();
            } else {
                break;
            }
        }

        Ok(node)
    }

    fn parse_subroutine_body(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::SubroutineBody);

        node.add_token(self.expect_symbol("{")?);

        while matches!(self.token.kind, Kind::Keyword("var")) {
            node.add_node(self.parse_var_dec("var")?);
        }

        node.add_node(self.parse_statements()?);

        node.add_token(self.expect_symbol("}")?);

        Ok(node)
    }

    fn parse_statements(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::Statements);

        loop {
            match self.token.kind {
                Kind::Keyword("let") => node.add_node(self.parse_let_statement()?),
                Kind::Keyword("if") => node.add_node(self.parse_if_statement()?),
                Kind::Keyword("while") => node.add_node(self.parse_while_statement()?),
                Kind::Keyword("do") => node.add_node(self.parse_do_statement()?),
                Kind::Keyword("return") => node.add_node(self.parse_return_statement()?),
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_let_statement(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::LetStatement);
        node.add_token(self.expect_keyword(&["let"])?);
        node.add_token(self.expect_ident()?);

        // Handle array index assignment syntax
        if matches!(self.token.kind, Kind::Symbol("[")) {
            node.add_token(self.expect_symbol("[")?);
            node.add_node(self.parse_expression()?);
            node.add_token(self.expect_symbol("]")?);
        }

        node.add_token(self.expect_symbol("=")?);
        node.add_node(self.parse_expression()?);
        node.add_token(self.expect_symbol(";")?);

        Ok(node)
    }

    fn parse_if_statement(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::IfStatement);

        node.add_token(self.expect_keyword(&["if"])?);
        node.add_token(self.expect_symbol("(")?);
        node.add_node(self.parse_expression()?);
        node.add_token(self.expect_symbol(")")?);
        node.add_token(self.expect_symbol("{")?);
        node.add_node(self.parse_statements()?);
        node.add_token(self.expect_symbol("}")?);

        if matches!(self.token.kind, Kind::Keyword("else")) {
            node.add_token(self.expect_keyword(&["else"])?);
            node.add_token(self.expect_symbol("{")?);
            node.add_node(self.parse_statements()?);
            node.add_token(self.expect_symbol("}")?);
        }

        Ok(node)
    }

    fn parse_while_statement(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::WhileStatement);

        node.add_token(self.expect_keyword(&["while"])?);
        node.add_token(self.expect_symbol("(")?);
        node.add_node(self.parse_expression()?);
        node.add_token(self.expect_symbol(")")?);
        node.add_token(self.expect_symbol("{")?);
        node.add_node(self.parse_statements()?);
        node.add_token(self.expect_symbol("}")?);

        Ok(node)
    }

    fn parse_do_statement(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::DoStatement);

        node.add_token(self.expect_keyword(&["do"])?);
        self.parse_subroutine_call(&mut node)?;
        node.add_token(self.expect_symbol(";")?);

        Ok(node)
    }

    fn parse_return_statement(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::ReturnStatement);
        node.add_token(self.expect_keyword(&["return"])?);

        if !matches!(self.token.kind, Kind::Symbol(";")) {
            node.add_node(self.parse_expression()?);
        }
        node.add_token(self.expect_symbol(";")?);

        Ok(node)
    }

    fn parse_expression(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::Expression);

        node.add_node(self.parse_term()?);

        loop {
            match self.token.kind {
                Kind::Symbol("+" | "-" | "*" | "/" | "&" | "|" | "<" | ">" | "=") => {
                    node.add_token(self.token);
                    self.advance();

                    node.add_node(self.parse_term()?);
                }
                _ => break,
            }
        }

        Ok(node)
    }

    fn parse_term(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::Term);

        match self.token.kind {
            // integerConstant
            Kind::IntConst(_)
            // stringConstant
            | Kind::StrConst(_)
            // keywordConstant
            | Kind::Keyword("true" | "false" | "null" | "this") => {
                node.add_token(self.token);
                self.advance();
            }
            // varName
            // varName '[' expression ']'
            // subroutineCall
            Kind::Ident(_) => {
                match self.peek().kind {
                    Kind::Symbol("[") => { 
                        node.add_token(self.expect_ident()?);
                        node.add_token(self.expect_symbol("[")?);
                        node.add_node(self.parse_expression()?);
                        node.add_token(self.expect_symbol("]")?);
                    }
                    Kind::Symbol("(" | ".") => {
                        self.parse_subroutine_call(&mut node)?;
                    }
                    _ => {
                        node.add_token(self.expect_ident()?);
                    }
                }
            }
            // '(' expression ')'
            Kind::Symbol("(") => {
                node.add_token(self.expect_symbol("(")?);
                node.add_node(self.parse_expression()?);
                node.add_token(self.expect_symbol(")")?);
            }
            // unaryOp term
            Kind::Symbol("-" | "~") => {
                node.add_token(self.token);
                self.advance();

                node.add_node(self.parse_term()?);
            }
            _ => return Err(self.unexpected_token_error("term")),
        }

        Ok(node)
    }

    fn parse_subroutine_call(&mut self, parent: &mut Node<'a>) -> ParseResult<()> {
        // Usually we'd allocate a new node in a method like this, but there's
        // no `subroutineCall` node type in the parse tree spec, but we do want
        // to reuse this logic (across terms and do statements), so we add the
        // elements directly to a borrowed parent node.
        parent.add_token(self.expect_ident()?);
        if matches!(self.token.kind, Kind::Symbol(".")) {
            parent.add_token(self.expect_symbol(".")?);
            parent.add_token(self.expect_ident()?);
        }

        parent.add_token(self.expect_symbol("(")?);
        parent.add_node(self.parse_expression_list()?);
        parent.add_token(self.expect_symbol(")")?);

        Ok(())
    }

    fn parse_expression_list(&mut self) -> NodeResult<'a> {
        let mut node = Node::new(NodeKind::ExpressionList);

        while !matches!(self.token.kind, Kind::Symbol(")")) {
            node.add_node(self.parse_expression()?);

            if matches!(self.token.kind, Kind::Symbol(",")) {
                node.add_token(self.token);
                self.advance();
            } else {
                break;
            }
        }

        Ok(node)
    }

    fn advance(&mut self) -> Token {
        self.prev_token = self.token;
        match self.peeked_token {
            Some(token) => {
                self.token = token;
                self.peeked_token = None;
            }
            None => {
                self.token = self.next_token();
            }
        }
        self.token
    }

    fn peek(&mut self) -> Token<'a> {
        match self.peeked_token {
            Some(token) => token,
            None => {
                self.peeked_token = Some(self.next_token());
                self.peeked_token.unwrap()
            }
        }
    }

    fn next_token(&mut self) -> Token<'a> {
        let mut token = self.tokenizer.next_token();
        while matches!(token.kind, Kind::Comment(_)) {
            token = self.tokenizer.next_token();
        }
        token
    }

    fn expect_type_name(&mut self) -> ParseResult<Token<'a>> {
        match self.token.kind {
            Kind::Keyword("void" | "int" | "char" | "boolean") | Kind::Ident(_) => {
                self.advance();
                Ok(self.prev_token)
            }
            _ => Err(self.unexpected_token_error("type")),
        }
    }

    fn expect_keyword(&mut self, allowed_values: &[&str]) -> ParseResult<Token<'a>> {
        match self.token {
            Token {
                kind: Kind::Keyword(lit),
                ..
            } if allowed_values.contains(&lit) => {
                self.advance();
                Ok(self.prev_token)
            }
            _ => Err(self.unexpected_token_error(&format!("keyword {:?}", allowed_values))),
        }
    }

    fn expect_symbol(&mut self, lit: &str) -> ParseResult<Token<'a>> {
        match self.token {
            Token {
                kind: Kind::Symbol(sym_lit),
                ..
            } if sym_lit == lit => {
                self.advance();
                Ok(self.prev_token)
            }
            _ => Err(self.unexpected_token_error(&format!("symbol {}", lit))),
        }
    }

    fn expect_ident(&mut self) -> ParseResult<Token<'a>> {
        match self.token {
            Token {
                kind: Kind::Ident(_),
                ..
            } => {
                self.advance();
                Ok(self.prev_token)
            }
            _ => Err(self.unexpected_token_error(&format!("identifier"))),
        }
    }

    fn span_error(&self, msg: String, span: Span) -> SpanError {
        SpanError::new(msg, span)
    }

    fn unexpected_token_error(&self, expected: &str) -> SpanError {
        let msg = format!(
            "unexpected token `{}', expected {}",
            self.token.kind, expected
        );
        self.span_error(msg, self.token.span)
    }
}

#[cfg(test)]
mod tests {
    use crate::jack::debugxml;

    use super::*;

    fn parse(src: &str) -> Element {
        Parser::new(Tokenizer::new(src)).parse().unwrap()
    }

    fn parse_tree_xml(src: &str) -> String {
        let mut buf = Vec::<u8>::new();
        debugxml::write_tree(&mut buf, &parse(src), 6);
        String::from_utf8(buf).unwrap()
    }

    #[test]
    fn test_empty_class() {
        let src = "
        class Foo {
        }
        ";
        let expected = "
        <class>
          <keyword> class </keyword>
          <identifier> Foo </identifier>
          <symbol> { </symbol>
          <symbol> } </symbol>
        </class>
        ";
        assert_eq!(
            normalize_whitespace(parse_tree_xml(src)),
            normalize_whitespace(expected)
        )
    }

    #[test]
    fn test_class_var_decs() {
        let src = "
        class Foo {
            field int x, y;
            static Point p;
        }
        ";
        let expected = "
        <class>
          <keyword> class </keyword>
          <identifier> Foo </identifier>
          <symbol> { </symbol>
          <classVarDec>
            <keyword> field </keyword>
            <keyword> int </keyword>
            <identifier> x </identifier>
            <symbol> , </symbol>
            <identifier> y </identifier>
            <symbol> ; </symbol>
          </classVarDec>
          <classVarDec>
            <keyword> static </keyword>
            <identifier> Point </identifier>
            <identifier> p </identifier>
            <symbol> ; </symbol>
          </classVarDec>
          <symbol> } </symbol>
        </class>
        ";
        assert_eq!(
            normalize_whitespace(parse_tree_xml(src)),
            normalize_whitespace(expected)
        )
    }

    #[test]
    fn test_functions() {
        let src = "
        class Foo {
            constructor Foo new(boolean x) {
            }
            
            method void bar(int x, String y) {
            }
        }
        ";
        let expected = "
        <class>
          <keyword> class </keyword>
          <identifier> Foo </identifier>
          <symbol> { </symbol>
          <subroutineDec>
            <keyword> constructor </keyword>
            <identifier> Foo </identifier>
            <identifier> new </identifier>
            <symbol> ( </symbol>
            <parameterList>
              <keyword> boolean </keyword>
              <identifier> x </identifier>
            </parameterList>
            <symbol> ) </symbol>
            <subroutineBody>
              <symbol> { </symbol>
              <statements>
              </statements>
              <symbol> } </symbol>
            </subroutineBody>
          </subroutineDec>
          <subroutineDec>
            <keyword> method </keyword>
            <keyword> void </keyword>
            <identifier> bar </identifier>
            <symbol> ( </symbol>
            <parameterList>
              <keyword> int </keyword>
              <identifier> x </identifier>
              <symbol> , </symbol>
              <identifier> String </identifier>
              <identifier> y </identifier>
            </parameterList>
            <symbol> ) </symbol>
            <subroutineBody>
              <symbol> { </symbol>
              <statements>
              </statements>
              <symbol> } </symbol>
            </subroutineBody>
          </subroutineDec>
          <symbol> } </symbol>
        </class>
        ";
        assert_eq!(
            normalize_whitespace(parse_tree_xml(src)),
            normalize_whitespace(expected)
        )
    }

    #[test]
    fn test_statements() {
        let src = "
        class Foo {
            function void bar() {
                var int x;
                if (true) {
                    do Sys.print(\"hi\");
                } else {
                    let x[1] = x;
                }
                while (false) {
                    return 1;
                }
                return;
            }
        }
        ";
        let expected = "
        <class>
          <keyword> class </keyword>
          <identifier> Foo </identifier>
          <symbol> { </symbol>
          <subroutineDec>
            <keyword> function </keyword>
            <keyword> void </keyword>
            <identifier> bar </identifier>
            <symbol> ( </symbol>
            <parameterList>
            </parameterList>
            <symbol> ) </symbol>
            <subroutineBody>
              <symbol> { </symbol>
              <varDec>
                <keyword> var </keyword>
                <keyword> int </keyword>
                <identifier> x </identifier>
                <symbol> ; </symbol>
              </varDec>
              <statements>
                <ifStatement>
                  <keyword> if </keyword>
                  <symbol> ( </symbol>
                  <expression>
                    <term>
                      <keyword> true </keyword>
                    </term>
                  </expression>
                  <symbol> ) </symbol>
                  <symbol> { </symbol>
                  <statements>
                    <doStatement>
                      <keyword> do </keyword>
                      <identifier> Sys </identifier>
                      <symbol> . </symbol>
                      <identifier> print </identifier>
                      <symbol> ( </symbol>
                      <expressionList>
                        <expression>
                          <term>
                            <stringConstant> hi </stringConstant>
                          </term>
                        </expression>
                      </expressionList>
                      <symbol> ) </symbol>
                      <symbol> ; </symbol>
                    </doStatement>
                  </statements>
                  <symbol> } </symbol>
                  <keyword> else </keyword>
                  <symbol> { </symbol>
                  <statements>
                    <letStatement>
                      <keyword> let </keyword>
                      <identifier> x </identifier>
                      <symbol> [ </symbol>
                      <expression>
                        <term>
                          <integerConstant> 1 </integerConstant>
                        </term>
                      </expression>
                      <symbol> ] </symbol>
                      <symbol> = </symbol>
                      <expression>
                        <term>
                          <identifier> x </identifier>
                        </term>
                      </expression>
                      <symbol> ; </symbol>
                    </letStatement>
                  </statements>
                  <symbol> } </symbol>
                </ifStatement>
                <whileStatement>
                  <keyword> while </keyword>
                  <symbol> ( </symbol>
                  <expression>
                    <term>
                      <keyword> false </keyword>
                    </term>
                  </expression>
                  <symbol> ) </symbol>
                  <symbol> { </symbol>
                  <statements>
                    <returnStatement>
                      <keyword> return </keyword>
                      <expression>
                        <term>
                          <integerConstant> 1 </integerConstant>
                        </term>
                      </expression>
                      <symbol> ; </symbol>
                    </returnStatement>
                  </statements>
                  <symbol> } </symbol>
                </whileStatement>
                <returnStatement>
                  <keyword> return </keyword>
                  <symbol> ; </symbol>
                </returnStatement>
              </statements>
              <symbol> } </symbol>
            </subroutineBody>
          </subroutineDec>
          <symbol> } </symbol>
        </class>
        ";
        assert_eq!(
            normalize_whitespace(parse_tree_xml(src)),
            normalize_whitespace(expected)
        )
    }

    #[test]
    fn test_expressions() {
        let src = "
        class Foo {
            function void bar() {
                let x = 1;
                let x = ~1;
                let x = \"hello\";
                let x = true;
                let x = x;
                let x = x[1];
                let x = 1 + 2 & (3 / 4);
                let x = baz(1);
                let x = Foo.quux(1, 2);
            }
        }
        ";
        let expected = "
        <class>
          <keyword> class </keyword>
          <identifier> Foo </identifier>
          <symbol> { </symbol>
          <subroutineDec>
            <keyword> function </keyword>
            <keyword> void </keyword>
            <identifier> bar </identifier>
            <symbol> ( </symbol>
            <parameterList>
            </parameterList>
            <symbol> ) </symbol>
            <subroutineBody>
              <symbol> { </symbol>
              <statements>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <integerConstant> 1 </integerConstant>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <symbol> ~ </symbol>
                      <term>
                        <integerConstant> 1 </integerConstant>
                      </term>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <stringConstant> hello </stringConstant>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <keyword> true </keyword>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <identifier> x </identifier>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <identifier> x </identifier>
                      <symbol> [ </symbol>
                      <expression>
                        <term>
                          <integerConstant> 1 </integerConstant>
                        </term>
                      </expression>
                      <symbol> ] </symbol>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <integerConstant> 1 </integerConstant>
                    </term>
                    <symbol> + </symbol>
                    <term>
                      <integerConstant> 2 </integerConstant>
                    </term>
                    <symbol> &amp; </symbol>
                    <term>
                      <symbol> ( </symbol>
                      <expression>
                        <term>
                          <integerConstant> 3 </integerConstant>
                        </term>
                        <symbol> / </symbol>
                        <term>
                          <integerConstant> 4 </integerConstant>
                        </term>
                      </expression>
                      <symbol> ) </symbol>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <identifier> baz </identifier>
                      <symbol> ( </symbol>
                      <expressionList>
                        <expression>
                          <term>
                            <integerConstant> 1 </integerConstant>
                          </term>
                        </expression>
                      </expressionList>
                      <symbol> ) </symbol>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
                <letStatement>
                  <keyword> let </keyword>
                  <identifier> x </identifier>
                  <symbol> = </symbol>
                  <expression>
                    <term>
                      <identifier> Foo </identifier>
                      <symbol> . </symbol>
                      <identifier> quux </identifier>
                      <symbol> ( </symbol>
                      <expressionList>
                        <expression>
                          <term>
                            <integerConstant> 1 </integerConstant>
                          </term>
                        </expression>
                        <symbol> , </symbol>
                        <expression>
                          <term>
                            <integerConstant> 2 </integerConstant>
                          </term>
                        </expression>
                      </expressionList>
                      <symbol> ) </symbol>
                    </term>
                  </expression>
                  <symbol> ; </symbol>
                </letStatement>
              </statements>
              <symbol> } </symbol>
            </subroutineBody>
          </subroutineDec>
          <symbol> } </symbol>
        </class>
        ";

        assert_eq!(
            normalize_whitespace(parse_tree_xml(src)),
            normalize_whitespace(expected)
        )
    }

    fn normalize_whitespace<S: AsRef<str>>(s: S) -> String {
        let lines = s
            .as_ref()
            .lines()
            .filter(|l| l.find(|c: char| !c.is_whitespace()).is_some());

        let min_indent = lines
            .clone()
            .map(|l| l.find(|c: char| !c.is_whitespace()).unwrap_or(0))
            .min()
            .unwrap_or(0);

        lines
            .map(|l| l[min_indent..].to_owned())
            .collect::<Vec<_>>()
            .join("\n")
    }
}
