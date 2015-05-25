extern crate docopt;

use docopt::Docopt;

fn main() {
	const VERSION: &'static str = env!("CARGO_PKG_VERSION");
	static USAGE: &'static str = "
	Usage:
	  sassers [options]

	Options:
	    -h, --help         Show this message.
	    -v, --version      Show the version of sassers.
	";

	let args = Docopt::new(USAGE)
	                  .and_then(|d| d.parse())
	                  .unwrap_or_else(|e| e.exit());

	if args.get_bool("-v") {
		println!("{}", VERSION);
	}
}
