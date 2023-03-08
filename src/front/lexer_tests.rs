use super::lexer;
use lexer::Token;

macro_rules! lex {
    ($l:literal) => {
        lexer::lex(String::from($l)).unwrap()
    };
}

fn vec_eq<T>(vec1: Vec<T>, vec2: Vec<T>) -> bool
where
    T: PartialEq,
{
    vec1.iter().zip(vec2).all(|(a, b)| *a == b)
}

#[test]
fn parentesis_test() {
    let res = lexer::lex(String::from("(){}[]")).unwrap();
    let real = vec![
        Token::ParenOpen,
        Token::ParenClose,
        Token::CurlyOpen,
        Token::CurlyClose,
        Token::BrackOpen,
        Token::BrackClose,
    ];
    assert!(vec_eq(res, real));
}

#[test]
fn equals_test() {
    let res = lexer::lex(String::from("+ ++ += - -- -= /= *=")).unwrap();
    let real = vec![
        Token::Plus,
        Token::PlusPlus,
        Token::PlusEquals,
        Token::Minus,
        Token::MinusMinus,
        Token::MinusEquals,
        Token::SlashEquals,
        Token::StarEquals,
    ];
    // println!("{:?}", res);
    assert!(vec_eq(res, real));
}
#[test]
fn logical() {
    let res = lexer::lex(String::from("||&&|& ")).unwrap();
    let real = vec![Token::LogicalOr, Token::LogicalAnd, Token::Or, Token::And];
    assert!(vec_eq(res, real));
}

#[test]
fn no_spaces() {
    let res = lexer::lex(String::from("=>=! =")).unwrap();
    let real = vec![Token::Arrow, Token::Equals, Token::Not, Token::Equals];
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
        Token::Arrow,
        Token::Equals,
        Token::StarEquals,
        Token::SlashEquals,
        Token::MinusEquals,
        Token::PlusEquals,
        Token::Or,
        Token::And,
        Token::LogicalOr,
        Token::LogicalAnd,
        Token::Eq,
        Token::Neq,
        Token::Leq,
        Token::Less,
        Token::Geq,
        Token::Greater,
        Token::ShiftLeft,
        Token::ShiftRight,
        Token::Plus,
        Token::Minus,
        Token::Times,
        Token::Slash,
        Token::Percent,
        Token::Power,
        Token::Not,
        Token::PlusPlus,
        Token::MinusMinus,
    ];
    assert!(vec_eq(res, real));
}

#[test]
fn simple_number() {
    let res = lexer::lex(String::from("420")).unwrap();
    let real = vec![Token::I64Literal(420)];
    assert!(vec_eq(res, real));

    let res = lex!("042");
    let real = vec![Token::I64Literal(42)];
    assert!(vec_eq(res, real));

    let res = lex!("4.2=");
    let real = vec![Token::F64Literal(4.2), Token::Equals];
    assert!(vec_eq(res, real));
}

#[test]
fn comments_simple() {
    let res = lex!("// Hello \n =");
    let real = vec![Token::Equals];
    assert!(vec_eq(res, real));
}

#[test]
fn comments_mutl_line() {
    let res = lex!("/**/ =");
    let real = vec![Token::Equals];
    assert!(vec_eq(res, real));
}

#[test]
fn hex_bin_literal() {
    let res = lex!("0xff");
    let real = vec![Token::I64Literal(0xff)];
    assert!(vec_eq(res, real));

    let res = lex!("0b10");
    let real = vec![Token::I64Literal(0b10)];
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
    let real = vec![Token::StringLiteral(String::from("Hello World"))];
    // println!("{:?}", res);
    assert!(vec_eq(res, real));
}

#[test]
fn string_escape() {
    let res = lex!(" \" \\n \\t \\r \\\\ \"");
    let real = vec![Token::StringLiteral(String::from(" \n \t \r \\ "))];
    println!("{:?}", res);
    assert!(vec_eq(res, real));
}

#[test]
fn keywords() {
    let res = lex!(
        "struct self fn let while Fn new int float bool str if else return import or and null"
    );
    let real = vec![
        Token::Keyword("struct"),
        Token::Keyword("self"),
        Token::Keyword("fn"),
        Token::Keyword("let"),
        Token::Keyword("while"),
        Token::Keyword("Fn"),
        Token::Keyword("new"),
        Token::Keyword("int"),
        Token::Keyword("float"),
        Token::Keyword("bool"),
        Token::Keyword("str"),
        Token::Keyword("if"),
        Token::Keyword("else"),
        Token::Keyword("return"),
        Token::Keyword("import"),
        Token::Keyword("or"),
        Token::Keyword("and"),
        Token::Keyword("null"),
    ];
    println!("{:?}", res);
    assert!(vec_eq(res, real));
}
