pub fn compile(sass: String, style: &str) -> String {
    let parsed = SassReader::new(sass).parse();
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

struct SassReader {
    pub pos: u32,
    pub last_pos: u32,
    pub curr: Option<char>,
    sass: String,
}

impl SassReader {
    pub fn new(str: String) -> SassReader {
        SassReader {
            pos: 0,
            last_pos: 0,
            curr: Some('\n'),
            sass: str,
        }
    }

    pub fn parse(self) -> String {
        self.sass
    }
}