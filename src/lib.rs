mod token;

pub fn compile(sass: &str, style: &str) -> Result<(), &'static str> {
    let mut sp = SassParser::new(&sass);
    let parsed = try!(sp.parse());
    println!("{:?}", parsed);
    Ok(())
    // match style {
    //     "nested"     => Ok(parsed.nested(&sp)),
    //     "compressed" => Ok(parsed.compressed(&sp)),
    //     "expanded"   => Ok(parsed.expanded(&sp)),
    //     "compact"    => Ok(parsed.compact(&sp)),
    //     _            => Err("Unknown style:. Please specify one of nested, compressed, expanded, or compact."),
    // }
}

enum Rule {
    SassRule,
}

enum Event {
    Start(Rule),
    End(Rule),
}

#[derive(Debug)]
struct SassParser<'a> {
    pub tokenizer: SassTokenizer<'a>,
    sass: &'a str,
}

impl<'a> SassParser<'a> {
    pub fn new(sass: &'a str) -> SassParser<'a> {
        let mut tokenizer = SassTokenizer::new(&sass);
        SassParser {
            tokenizer: tokenizer,
            sass: &sass,
        }
    }

    pub fn parse(&mut self) -> Result<(), &'static str> {
        while self.tokenizer.next().is_some() { }

        Ok(())
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
        SassTokenizer {
            pos: 0,
            last_pos: 0,
            curr: Some('\n'),
            peek_range: token::Range { start_pos: 0, end_pos: 0, token: token::Eof },
            sass: &sass,
        }
    }
}

impl<'a> Iterator for SassTokenizer<'a> {
    type Item = Event;

    fn next(&mut self) -> Option<Event> {
        println!("totes got here");
        None
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
