#![feature(collections)]
#![feature(convert)]
mod token;

pub fn compile(sass: &str, style: &str) -> Result<String, &'static str> {
    let mut sp = SassParser::new(&sass);
    let parsed = try!(sp.parse());
    match style {
        "nested"     => Ok(parsed.nested(&sp)),
        "compressed" => Ok(parsed.compressed(&sp)),
        "expanded"   => Ok(parsed.expanded(&sp)),
        "compact"    => Ok(parsed.compact(&sp)),
        _            => Err("Unknown style:. Please specify one of nested, compressed, expanded, or compact."),
    }
}

#[derive(Debug)]
struct SassRuleSet {
    pub rules: Vec<SassRule>,
}

impl SassRuleSet {
    fn nested(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");
        for rule in &self.rules {
            output.push_str((&rule).nested(&sp).as_str());
        }
        output
    }

    fn compressed(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");
        for rule in &self.rules {
            output.push_str((&rule).compressed(&sp).as_str());
        }
        output
    }

    fn expanded(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");
        for rule in &self.rules {
            output.push_str((&rule).expanded(&sp).as_str());
        }
        output
    }

    fn compact(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");
        for rule in &self.rules {
            output.push_str((&rule).compact(&sp).as_str());
        }
        output
    }
}

#[derive(Debug)]
struct SassRule {
    pub selectors: Vec<token::Range>,
    pub props_and_values: Vec<PropertyValueSet>,
    pub subrule_set: SassRuleSet,
}

impl SassRule {
    fn nested(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");

        for selector in &self.selectors {
            output.push_str(&sp.extract(selector));
        }

        output.push_str(" {");

        for prop_and_val in &self.props_and_values {
            output.push_str(format!("\n  {}: {};", &sp.extract(&prop_and_val.property), &sp.extract(&prop_and_val.value)).as_str());
        }

        output.push_str(" }\n");

        output
    }

    fn compressed(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");

        for selector in &self.selectors {
            output.push_str(&sp.extract(selector));
        }

        output.push_str("{");

        for prop_and_val in &self.props_and_values {
            output.push_str(format!("{}:{}", &sp.extract(&prop_and_val.property), &sp.extract(&prop_and_val.value)).as_str());
        }

        output.push_str("}\n");

        output
    }

    fn expanded(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");

        for selector in &self.selectors {
            output.push_str(&sp.extract(selector));
        }

        output.push_str(" {");

        for prop_and_val in &self.props_and_values {
            output.push_str(format!("\n  {}: {};", &sp.extract(&prop_and_val.property), &sp.extract(&prop_and_val.value)).as_str());
        }

        output.push_str("\n}\n");

        output
    }

    fn compact(&self, sp: &SassParser) -> String {
        let mut output =  String::from_str("");

        for selector in &self.selectors {
            output.push_str(&sp.extract(selector));
        }

        output.push_str(" {");

        for prop_and_val in &self.props_and_values {
            output.push_str(format!(" {}: {};", &sp.extract(&prop_and_val.property), &sp.extract(&prop_and_val.value)).as_str());
        }

        output.push_str(" }\n");

        output
    }
}

#[derive(Debug)]
struct PropertyValueSet {
    pub property: token::Range,
    pub value: token::Range,
}

#[derive(Debug)]
struct SassParser<'a> {
    pub token: token::Range,
    pub last_token: token::Range,
    pub tokenizer: SassTokenizer<'a>,
    sass: &'a str,
}

impl<'a> SassParser<'a> {
    pub fn new(sass: &'a str) -> SassParser<'a> {
        let mut tokenizer = SassTokenizer::new(&sass);
        let initial_token = tokenizer.real_token();
        SassParser {
            token: initial_token.clone(),
            last_token: initial_token.clone(),
            tokenizer: tokenizer,
            sass: &sass,
        }
    }

    pub fn extract(&self, range: &token::Range) -> &str {
        self.sass.slice_chars(range.start_pos as usize, range.end_pos as usize)
    }

    pub fn bump(&mut self) {
        self.last_token = self.token.clone();
        self.token = self.tokenizer.real_token();
    }

    pub fn parse(&mut self) -> Result<SassRuleSet, &'static str> {
        let mut rules = vec![];
        while let Some(rule) = try!(self.parse_rule()) {
            rules.push(rule);
        }
        Ok(SassRuleSet { rules: rules })
    }

    fn parse_rule(&mut self) -> Result<Option<SassRule>, &'static str> {
        if self.token.token == token::Eof {
            return Ok(None)
        }

        let mut selectors = vec![];
        while let Some(selector) = try!(self.parse_selector()) {
            selectors.push(selector);
        }
        if selectors.len() == 0 {
            return Err("Empty selector!")
        }

        let mut props_and_values = vec![];
        let mut subrules = vec![];
        while let Some(property_value_set_or_subrule) = try!(self.parse_property_value_set_or_subrule()) {
            props_and_values.push(property_value_set_or_subrule);
        }

        // A rule without properties and values isn't *wrong*, per se...
        Ok(Some(SassRule { selectors: selectors, props_and_values: props_and_values, subrule_set: SassRuleSet { rules: subrules }}))
    }

    fn parse_selector(&mut self) -> Result<Option<token::Range>, &'static str> {
        if self.token.token == token::Eof {
            return Ok(None)
        }

        if self.token.token == token::Text {
            self.bump();
            return Ok(Some(self.last_token.clone()))
        }

        match &self.token.token {
            &token::OpenDelim(token::Brace) => {
                self.bump();
                Ok(None)
            },
            &token::Comma => {
                self.bump();
                self.parse_selector()
            },
            _ => Err("Unexpected token where we expected selectors"),
        }
    }

    fn parse_property_value_set_or_subrule(&mut self) -> Result<Option<PropertyValueSet>, &'static str> {
        if self.token.token == token::Eof {
            return Ok(None)
        }

        if self.token.token == token::CloseDelim(token::Brace) {
            self.bump();
            return Ok(None)
        }

        let p = self.token.clone();

        match p.token {
            token::Text => {
                self.bump(); // should be colon
                self.bump(); // optional whitespace required for now

                let v = self.token.clone();

                if v.token == token::Text {
                    self.bump(); // semicolon
                    self.bump(); // whitespace
                    return Ok(Some(PropertyValueSet { property: p, value: v }))
                } else {
                    return Err("Expected a value here, instead got who knows what!")
                }
            },
            token::CloseDelim(token::Brace) => {
                self.bump();
                Ok(None)
            },
            _ => Err("Unexpected token where we expected properties and values"),
        }
    }
}

#[derive(Debug)]
struct SassTokenizer<'a> {
    pub pos: u32,
    pub last_pos: u32,
    pub curr: Option<char>,
    pub peek_range: token::Range,
    sass: &'a str,
}

impl<'a> SassTokenizer<'a> {
    pub fn new(sass: &'a str) -> SassTokenizer<'a> {
        let mut sr = SassTokenizer {
            pos: 0,
            last_pos: 0,
            curr: Some('\n'),
            peek_range: token::Range { start_pos: 0, end_pos: 0, token: token::Eof },
            sass: &sass,
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

    fn advance_token(&mut self) {
        match self.scan_whitespace() {
            Some(whitespace) => {
                self.peek_range = whitespace;
            },
            None => {
                if self.is_eof() {
                    self.peek_range = token::Range { start_pos: self.pos + 1, end_pos: self.pos + 1, token: token::Eof };
                } else {
                    self.peek_range = self.next_token_inner();
                }
            },
        }
    }

    fn next_token(&mut self) -> token::Range {
        let retval = self.peek_range.clone();
        self.advance_token();
        retval
    }

    fn next_token_inner(&mut self) -> token::Range {
        let c = match self.curr {
            Some(c) => c,
            None => return token::Range { start_pos: self.pos + 1, end_pos: self.pos + 1, token: token::Eof },
        };

        if c >= 'a' && c <= 'z' {
            let start = self.last_pos;
            while !self.curr.is_none() && self.curr.unwrap() >= 'a' && self.curr.unwrap() <= 'z' {
                self.bump();
            }
            return token::Range { start_pos: start, end_pos: self.last_pos, token: token::Text }
        }

        match c {
            ';' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Semi }; },
            ':' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Colon }; },
            ',' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Comma }; },
            '{' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::OpenDelim(token::Brace) }; },
            '}' => { self.bump(); return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::CloseDelim(token::Brace) }; },
            _   => { return token::Range { start_pos: self.last_pos, end_pos: self.last_pos, token: token::Unknown} },
        }
    }

    fn real_token(&mut self) -> token::Range {
        let mut t = self.next_token();
        loop {
            match t.token {
                token::Whitespace => {
                    t = self.next_token();
                },
                _ => break,
            }
        }
        t
    }

    fn scan_whitespace(&mut self) -> Option<token::Range> {
        match self.curr.unwrap_or('\0') {
            c if is_whitespace(Some(c)) => {
                let start = self.last_pos;
                while is_whitespace(self.curr) { self.bump(); }
                Some(token::Range { start_pos: start, end_pos: self.last_pos, token: token::Whitespace })
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
