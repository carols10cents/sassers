pub fn compile(sass: String, style: &str) -> String {
    match style {
        "nested"     => nested(sass),
        "compressed" => compressed(sass),
        "expanded"   => expanded(sass),
        "compact"    => compact(sass),
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
