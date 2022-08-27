use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub(crate) struct Args {
	/// Name of the person to greet
	#[clap(short, long)]
	system: bool,
}

pub(crate) fn parse_args() -> Args {
	Args::parse()
}