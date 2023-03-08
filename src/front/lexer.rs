use std::fs;

macro_rules! kw {
    ($sl:expr, $l:literal) => {
        $sl.push(Token::Keyword($l))
    } 
}

#[derive(Debug, PartialEq)]
pub enum Token {
    Keyword(&'static str),
    Identifier(String),
    ParenOpen,
    ParenClose,
    CurlyOpen,
    CurlyClose,
    BrackOpen,
    BrackClose,
    Coma,
    DoublePoint,
    Semicol,
    I64Literal(i64),
    F64Literal(f64),
    BoolLiteral(bool),
    StringLiteral(String),
    Arrow,  // =>
    Equals, // =
    StarEquals,
    SlashEquals,
    MinusEquals,
    PlusEquals,
    LogicalOr,
    LogicalAnd,
    Or,
    And,
    Eq,  // ==
    Neq, // !=
    Leq,
    Less,
    Geq,
    Greater,
    ShiftLeft,
    ShiftRight,
    Plus,
    Minus,
    Times,
    Slash,
    Percent,
    Power,
    Not,
    PlusPlus,
    MinusMinus,
}
#[derive(Debug)]
pub enum LangError {
    Unknown,
    LexingError,
}

pub fn lex_file(path: &String) -> Result<Vec<Token>, LangError> {
    let file = fs::read_to_string(path);
    if let Err(_) = file {
        return Err(LangError::Unknown);
    }
    let code = file.unwrap();

    lex(code)
}

pub fn lex(code: String) -> Result<Vec<Token>, LangError> {
    let mut lexer = Lexer::new(code);
    
    while lexer.current < lexer.chars.len() {
        match lexer.chars[lexer.current] {
            '(' => lexer.push(Token::ParenOpen),
            ')' => lexer.push(Token::ParenClose),
            '{' => lexer.push(Token::CurlyOpen),
            '}' => lexer.push(Token::CurlyClose),
            '[' => lexer.push(Token::BrackOpen),
            ']' => lexer.push(Token::BrackClose),
            ',' => lexer.push(Token::Coma),
            ':' => lexer.push(Token::DoublePoint),
            ';' => lexer.push(Token::Semicol),
            '%' => lexer.push(Token::Percent),
            '=' => lexer.equals(),
            '*' => lexer.star(),
            '/' => lexer.slash(),
            '-' => lexer.minus(),
            '+' => lexer.plus(),
            '&' => lexer.and(),
            '|' => lexer.or(),
            '!' => lexer.not(),
            '>' => lexer.greater(),
            '<' => lexer.less(),
            '1'..='9' => lexer.number(),
            '0' => lexer.hex_bin_number(),
            '"' => lexer.string_literal(),
            ' ' => {},
            '\n' | '\r' => lexer.line += 1,
            _ => lexer.keyword_ident(), 
        }
        if lexer.had_error {
            return Err(LangError::LexingError);
        }
        lexer.next();
    }

    return Ok(lexer.tokens);
}

struct Lexer {
    current: usize,
    chars: Vec<char>,
    tokens: Vec<Token>,
    had_error: bool,
    line: u32,
}

impl Lexer {
    fn new(code: String) -> Lexer {
        Lexer {
            current: 0,
            chars: code.chars().collect(),
            tokens: Vec::new(),
            had_error: false,
            line: 0,
        }
    }

    fn push(&mut self, tk: Token) {
        self.tokens.push(tk);
    }

    fn push_and_next(&mut self, tk: Token) {
        self.push(tk);
        self.next();
    }

    fn next(&mut self) {
        self.current += 1;
    }

    fn error(&mut self, message: &str) {
        self.had_error = true;
        println!("{}: Error {}", self.line, message);
    }

    fn peek(&self, amount: usize) -> Option<char> {
        if self.current + amount < self.chars.len() {
            Some(self.chars[self.current + amount])
        } else {
            None
        }
    }

    fn error_if_end(&mut self) -> bool {
        if self.current >= self.chars.len() {
            self.error("File ended unexpected");
            return true;
        }
        return false;
    }

    fn equals(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Equals);
            return;
        }
        match self.peek(1).unwrap_or('0') {
            '=' => self.push_and_next(Token::Eq),
            '>' => self.push_and_next(Token::Arrow),
            _ => self.push(Token::Equals),
        }
    }

    fn command_one_line(&mut self) {
        self.next();
        while self.current < self.chars.len() {
            match self.chars[self.current] {
                '\t' | '\n' => {
                    self.line += 1;
                    return;
                }
                _ => {}
            }
            self.next();
        }
    }

    fn command_mult_line(&mut self) {
        self.next();
        self.next();
        while self.current < self.chars.len() {
            match self.chars[self.current] {
                '*' => {
                    if self.peek(1).unwrap_or('0') == '/' {
                        self.next();
                        return;
                    }
                }
                _ => {}
            }
            self.next();
        }
        self.error("Lexer: Unclosed command");
    }

    fn star(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Times);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '*' => self.push_and_next(Token::Power),
            '=' => self.push_and_next(Token::StarEquals),
            _ => self.push(Token::Times),
        }
    }

    fn slash(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Slash);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '/' => self.command_one_line(),
            '=' => self.push_and_next(Token::SlashEquals),
            '*' => self.command_mult_line(),
            _ => self.push(Token::Slash),
        }
    }

    fn minus(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Minus);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '-' => self.push_and_next(Token::MinusMinus),
            '=' => self.push_and_next(Token::MinusEquals),
            _ => self.push(Token::Minus),
        }
    }

    fn plus(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Plus);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '+' => self.push_and_next(Token::PlusPlus),
            '=' => self.push_and_next(Token::PlusEquals),
            _ => self.push(Token::Plus),
        }
    }

    fn and(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::And);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '&' => self.push_and_next(Token::LogicalAnd),
            _ => self.push(Token::And),
        }
    }

    fn or(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Or);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '|' => self.push_and_next(Token::LogicalOr),
            _ => self.push(Token::Or),
        }
    }

    fn not(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Not);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '=' => self.push_and_next(Token::Neq),
            _ => self.push(Token::Not),
        }
    }

    fn greater(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Greater);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '>' => self.push_and_next(Token::ShiftRight),
            '=' => self.push_and_next(Token::Geq),
            _ => self.push(Token::Greater),
        }
    }

    fn less(&mut self) {
        if self.current >= self.chars.len() {
            self.push(Token::Less);
            return;
        }

        match self.peek(1).unwrap_or('0') {
            '<' => self.push_and_next(Token::ShiftLeft),
            '=' => self.push_and_next(Token::Leq),
            _ => self.push(Token::Less),
        }
    }

    // normal num   42  ok
    // negative num -42 ok
    // float num    4.2
    // hex          0xff
    // did not start with zero.
    fn number(&mut self) {
        let mut number = vec![self.chars[self.current]];
        while self.peek(1).unwrap_or('a').is_numeric() {
            self.next();
            number.push(self.chars[self.current]);
        }
        let num;
        if let Ok(ok_num) = number.into_iter().collect::<String>().parse::<i64>() {
            num = ok_num;
        } else {
            self.error("Lexer: Problem parsing number literal");
            return;
        }

        if self.peek(1).unwrap_or('a') == '.' {
            self.next();
            let mut decimals: Vec<char> = Vec::new();
            while self.peek(1).unwrap_or('a').is_numeric() {
                self.next();
                decimals.push(self.chars[self.current]);
            }

            let dec_num;
            if let Ok(ok_dec_num) =
                format!("{}.{}", num, decimals.into_iter().collect::<String>()).parse::<f64>()
            {
                dec_num = ok_dec_num;
            } else {
                self.error("Lexer: Problem parsing floating point number literal.");
                return;
            }
            self.push(Token::F64Literal(dec_num));
        } else {
            self.push(Token::I64Literal(num));
        }
    }

    fn hex_bin_number(&mut self) {
        match self.peek(1).unwrap_or('z') {
            'x' => self.hex(),
            'b' => self.bin(),
            _ => self.number(),
        }
    }

    fn hex(&mut self) {
        self.next();

        let mut digits: Vec<char> = Vec::new();

        while self.peek(1).unwrap_or('y').is_digit(16) {
            self.next();
            digits.push(self.chars[self.current]);
        }

        if let Ok(ok) = i64::from_str_radix(&digits.iter().collect::<String>(), 16) {
            self.push(Token::I64Literal(ok));
        } else {
            self.error("Lexer: Problem parsing hex literal.");
        }
    }

    fn bin(&mut self) {
        self.next();

        let mut digits: Vec<char> = Vec::new();

        while self.peek(1).unwrap_or('y').is_digit(2) {
            self.next();
            digits.push(self.chars[self.current]);
        }
        
        if let Ok(ok) = i64::from_str_radix(&digits.iter().collect::<String>(), 2) {
            self.push(Token::I64Literal(ok));
        } else {
            self.error("Lexer: Problem parsing bin literal");
        }
    }

    fn string_literal(&mut self) {
        let mut string = String::new();

        while let Some(x) = self.peek(1) {
            match x {
                // backslash character
                '\u{005C}' => {
                    if let Some(seq) = self.escape_seq() {
                        string.push(seq);
                    } else {
                        self.error("Lexer: Cannot parse escape_seq in string.");
                        return;
                    }
                }, 
                '"' => {
                    self.push(Token::StringLiteral(string));
                    self.next();
                    return;
                }
                _ => string.push(x) 
            }
            self.next();
        } 
        self.error("Lexer: Unclosed String literal.");
    }

    fn escape_seq(&mut self) -> Option<char> {
        self.next();
        match self.peek(1).unwrap_or('y') {
            't' => Some('\t'),
            'n' => Some('\n'),
            'r' => Some('\r'),
            '\u{005C}' => Some('\u{005C}'),
            '"' => Some('"'),
            '\'' => Some('\''),
            _ => None
        }
    }

    fn keyword_ident(&mut self) {
        // first get keyword, so parse until found no a-Z | 0-1
        let mut ident = String::new();
        ident.push(self.chars[self.current]);

        while let Some(c) = self.peek(1) {
            if c.is_alphabetic() || c.is_digit(10) || c == '_' {
                ident.push(c);
            } else {
                break;
            }
            self.next();
        }
        
        if ident.len() == 0 {
            self.error("Lexer: Invalid Token")
        }

        if ident.chars().nth(0).unwrap().is_digit(10) {
            self.error("Lexer: Identifier cannot start with number");
        }
        
        match ident.as_str() {
            "true"  => self.push(Token::BoolLiteral(true)),
            "false" => self.push(Token::BoolLiteral(false)),
            "struct" => kw!(self, "struct"),
            "self" => kw!(self, "self"),
            "fn" => kw!(self, "fn"),
            "let" => kw!(self, "let"),
            "while" => kw!(self, "while"),
            "Fn" => kw!(self, "Fn"),
            "new" => kw!(self, "new"),
            "int" => kw!(self, "int"),
            "float" => kw!(self, "float"),
            "bool" => kw!(self, "bool"),
            "str" => kw!(self, "str"),
            "if" => kw!(self, "if"),
            "else" => kw!(self, "else"),
            "return" => kw!(self, "return"),
            "import" => kw!(self, "import"),
            "or" => kw!(self, "or"),
            "and" => kw!(self, "and"),
            "null" => kw!(self, "null"),
            _ => self.push(Token::Identifier(ident)),
        }
    }
}
