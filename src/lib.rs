mod event;
mod output;
mod sass;
mod tokenizer;
mod top_level_event;
mod variable_mapper;

use tokenizer::Tokenizer;

pub fn compile(sass: &str, style: &str) -> Result<String, &'static str> {
    let mut st = Tokenizer::new(&sass);

    match style {
        "nested"     => Ok(output::nested(&mut st)),
        "compressed" => Ok(output::compressed(&mut st)),
        "expanded"   => Ok(output::expanded(&mut st)),
        "compact"    => Ok(output::compact(&mut st)),
        "debug"      => Ok(output::debug(&mut st)),
        _            => Err("Unknown style:. Please specify one of nested, compressed, expanded, or compact."),
    }
}

