pub fn compile(sass: String, style: &str) -> String {
    let sr = SassReader::new(sass);
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
    sass: String,
}

impl SassReader {
    pub fn new(str: String) -> SassReader {
        let mut sr = SassReader {
            pos: 0,
            last_pos: 0,
            curr: Some('\n'),
            sass: str,
        };
        sr.bump();
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

    pub fn parse(self) -> String {
        self.sass
    }
}

pub fn char_at(s: &str, byte: usize) -> char {
    s[byte..].chars().next().unwrap()
}
