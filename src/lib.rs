mod token;

pub fn compile(sass: String, style: &str) -> String {
    let mut sr = SassReader::new(sass);
    println!("{:?}", sr);
    let parsed = sr.parse();
    match style {
        "nested"     => nested(parsed),
        "compressed" => compressed(parsed),
        "expanded"   => expanded(parsed),
        "compact"    => compact(parsed),
        _            => panic!("Unknown style: {}. Please specify one of nested, compressed, expanded, or compact.", style),
    }
}

fn nested(sass: String) -> String {
    sass
}

fn compressed(sass: String) -> String {
    sass.replace(" ", "").replace("\n", "")
}

fn expanded(sass: String) -> String {
    sass
}

fn compact(sass: String) -> String {
    sass
}

#[derive(Debug)]
struct SassReader {
    pub pos: u32,
    pub last_pos: u32,
    pub curr: Option<char>,
    pub peek_tok: token::Token,
    sass: String,
}

impl SassReader {
    pub fn new(str: String) -> SassReader {
        let mut sr = SassReader {
            pos: 0,
            last_pos: 0,
            curr: Some('\n'),
            peek_tok: token::Eof,
            sass: str,
        };
        sr.bump();
        sr.advance_token();
        sr
    }

    pub fn bump(&mut self) {
        self.last_pos = self.pos;
        let current_pos = self.pos as usize;
        if current_pos < self.sass.len() {
            let ch = char_at(&self.sass, current_pos);

            self.pos = self.pos + (ch.len_utf8() as u32);
            self.curr = Some(ch);
        } else {
            self.curr = None;
        }
    }

    pub fn advance_token(&mut self) {
        match self.scan_whitespace() {
            Some(whitespace) => {
                self.peek_tok = whitespace;
            },
            None => {
                if self.is_eof() {
                    self.peek_tok = token::Eof;
                } else {
                    self.peek_tok = self.next_token_inner();
                }
            },
        }
    }

    pub fn next_token(&mut self) -> token::Token {
        let ret_val = self.peek_tok.clone();
        self.advance_token();
        ret_val
    }

    pub fn parse(&mut self) -> String {
        println!("{:?}", self.next_token());
        println!("{:?}", self.next_token());
        self.sass.clone()
    }

    fn next_token_inner(&mut self) -> token::Token {
        let c = match self.curr {
            Some(c) => c,
            None => return token::Eof,
        };

        if c >= 'a' && c <= 'z' {
            let start = self.last_pos;
            while !self.curr.is_none() && self.curr.unwrap() >= 'a' && self.curr.unwrap() <= 'z' {
                self.bump();
            }
            return token::Selector { start_pos: start, end_pos: self.last_pos }
        } else {
            token::Eof
        }

    }

    fn scan_whitespace(&mut self) -> Option<token::Token> {
        match self.curr.unwrap_or('\0') {
            c if is_whitespace(Some(c)) => {
                let start = self.last_pos;
                while is_whitespace(self.curr) { self.bump(); }
                Some(token::Whitespace { start_pos: start, end_pos: self.last_pos })
            },
            _ => None
        }
    }

    fn is_eof(&self) -> bool {
        self.curr.is_none()
    }
}

pub fn char_at(s: &str, byte: usize) -> char {
    s[byte..].chars().next().unwrap()
}

pub fn is_whitespace(c: Option<char>) -> bool {
    match c.unwrap_or('\x00') { // None can be null for now... it's not whitespace
        ' ' | '\n' | '\t' | '\r' => true,
        _ => false
    }
}
