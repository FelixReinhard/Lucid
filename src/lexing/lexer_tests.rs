use super::lexer;
use lexer::{TokenData, Token};

macro_rules! lex {
    ($l:literal) => {
        lexer::lex(String::from($l)).unwrap()
    };
}

fn vec_eq(vec1: Vec<Token>, vec2: Vec<TokenData>) -> bool {
    vec1.iter().zip(vec2).all(|(a, b)| a.tk == b)
}

#[test]
fn parentesis_test() {
    let res = lexer::lex(String::from("(){}[]")).unwrap();
    let real = vec![
        TokenData::ParenOpen,
        TokenData::ParenClose,
        TokenData::CurlyOpen,
        TokenData::CurlyClose,
        TokenData::BrackOpen,
        TokenData::BrackClose,
    ];
    assert!(vec_eq(res, real));
}

#[test]
fn equals_test() {
    let res = lexer::lex(String::from("+ ++ += - -- -= /= *=")).unwrap();
    let real = vec![
        TokenData::Plus,
        TokenData::PlusPlus,
        TokenData::PlusEquals,
        TokenData::Minus,
        TokenData::MinusMinus,
        TokenData::MinusEquals,
        TokenData::SlashEquals,
        TokenData::StarEquals,
    ];
    // println!("{:?}", res);
    assert!(vec_eq(res, real));
}
#[test]
fn logical() {
    let res = lexer::lex(String::from("||&&|& ")).unwrap();
    let real = vec![TokenData::LogicalOr, TokenData::LogicalAnd, TokenData::Or, TokenData::And];
    assert!(vec_eq(res, real));
}

#[test]
fn no_spaces() {
    let res = lexer::lex(String::from("=>=! =")).unwrap();
    let real = vec![TokenData::Arrow, TokenData::Equals, TokenData::Not, TokenData::Equals];
    // println!("{:?}", res);
    assert!(vec_eq(res, real));
}

#[test]
fn all_ops() {
    let res = lexer::lex(String::from(
        "=> = *= /= -= += | & || && == != <= < >= > << >>  + - * / % ** ! ++ --",
    ))
    .unwrap();
    let real = vec![
        TokenData::Arrow,
        TokenData::Equals,
        TokenData::StarEquals,
        TokenData::SlashEquals,
        TokenData::MinusEquals,
        TokenData::PlusEquals,
        TokenData::Or,
        TokenData::And,
        TokenData::LogicalOr,
        TokenData::LogicalAnd,
        TokenData::Eq,
        TokenData::Neq,
        TokenData::Leq,
        TokenData::Less,
        TokenData::Geq,
        TokenData::Greater,
        TokenData::ShiftLeft,
        TokenData::ShiftRight,
        TokenData::Plus,
        TokenData::Minus,
        TokenData::Times,
        TokenData::Slash,
        TokenData::Percent,
        TokenData::Power,
        TokenData::Not,
        TokenData::PlusPlus,
        TokenData::MinusMinus,
    ];
    assert!(vec_eq(res, real));
}

#[test]
fn simple_number() {
    let res = lexer::lex(String::from("420")).unwrap();
    let real = vec![TokenData::I64Literal(420)];
    assert!(vec_eq(res, real));

    let res = lex!("042");
    let real = vec![TokenData::I64Literal(42)];
    assert!(vec_eq(res, real));

    let res = lex!("4.2=");
    let real = vec![TokenData::F64Literal(4.2), TokenData::Equals];
    assert!(vec_eq(res, real));
}

#[test]
fn negative_number() {
    let res = lex!("-42");
    let real = vec![TokenData::Minus, TokenData::I64Literal(42)];
    assert!(vec_eq(res, real));
}

#[test]
fn comments_simple() {
    let res = lex!("// Hello \n =");
    let real = vec![TokenData::Equals];
    assert!(vec_eq(res, real));
}

#[test]
fn comments_mutl_line() {
    let res = lex!("/**/ =");
    let real = vec![TokenData::Equals];
    assert!(vec_eq(res, real));
}

#[test]
fn hex_bin_literal() {
    let res = lex!("0xff");
    let real = vec![TokenData::I64Literal(0xff)];
    assert!(vec_eq(res, real));

    let res = lex!("0b10");
    let real = vec![TokenData::I64Literal(0b10)];
    assert!(vec_eq(res, real));
}

#[test]
#[should_panic]
fn hex_bin_lit_fail() {
    let _ = lex!("0x");
    let _ = lex!("0b");
}
#[test]
fn string_literal() {
    let res = lex!("\"Hello World\" ");
    let real = vec![TokenData::StringLiteral(String::from("Hello World"))];
    // println!("{:?}", res);
    assert!(vec_eq(res, real));
}

#[test]
fn string_escape() {
    let res = lex!(" \" \\n \\t \\r \\\\ \"");
    let real = vec![TokenData::StringLiteral(String::from(" \n \t \r \\ "))];
    println!("{:?}", res);
    assert!(vec_eq(res, real));
}

#[test]
fn keywords() {
    let res = lex!(
        "struct self fn let while Fn new int float bool str if else return import or and null"
    );
    let real = vec![
        TokenData::Keyword("struct"),
        TokenData::Keyword("self"),
        TokenData::Keyword("fn"),
        TokenData::Keyword("let"),
        TokenData::Keyword("while"),
        TokenData::Keyword("Fn"),
        TokenData::Keyword("new"),
        TokenData::Keyword("int"),
        TokenData::Keyword("float"),
        TokenData::Keyword("bool"),
        TokenData::Keyword("str"),
        TokenData::Keyword("if"),
        TokenData::Keyword("else"),
        TokenData::Keyword("return"),
        TokenData::Keyword("import"),
        TokenData::Keyword("or"),
        TokenData::Keyword("and"),
        TokenData::Keyword("null"),
    ];
    println!("{:?}", res);
    assert!(vec_eq(res, real));
}
