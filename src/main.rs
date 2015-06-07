extern crate sassers;
extern crate docopt;

use docopt::Docopt;
use std::fs::File;
use std::io::Read;
use std::path::Path;

fn main() {
    const VERSION: &'static str = env!("CARGO_PKG_VERSION");
    static USAGE: &'static str = "
Usage:
    sassers [-t <style>] <inputfile>
    sassers [-vh]

Options:
    -h, --help                   Show this message
    -v, --version                Show the version
    -t <style>, --style <style>  Output style [default: nested]
    ";

    let args = Docopt::new(USAGE)
                      .and_then(|d| d.parse())
                      .unwrap_or_else(|e| e.exit());

    if args.get_bool("-v") {
        println!("{}", VERSION);
    } else {
        let style = args.get_str("-t");
        let inputfile = args.get_str("<inputfile>");

        let mut sass = String::new();
        let mut file = match File::open(&Path::new(&inputfile)) {
            Ok(f) => f,
            Err(msg) => panic!("File not found! {}", msg),
        };
        match file.read_to_string(&mut sass) {
            Ok(_) => {
                match sassers::compile(&sass, style) {
                    Ok(compiled) => println!("{}", compiled),
                    Err(msg) => println!("Compilation failed: {}", msg),
                }
            },
            Err(msg) => panic!("Could not read file! {}", msg),
        }
    }
}
