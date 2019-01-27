#[cfg(not(test))]
fn main() {
    use log::debug;
    use docopt::Docopt;
    env_logger::init();
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
        let input_filename = args.get_str("<inputfile>");
        debug!("input filename = {:?}", input_filename);

        sassers::compile(input_filename, &mut std::io::stdout(), style).unwrap_or_else(|e| {
            println!("Compilation failed: {}", e.message);
        });
    }
}
