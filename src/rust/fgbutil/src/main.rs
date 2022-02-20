use geozero::GeozeroDatasource;
use flatgeobuf::FgbReader;
use std::fs::File;
use std::io::BufReader;
use geozero::error::{Result};
use geozero::geojson::GeoJsonWriter;
use clap::Parser;

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input file
    #[clap(short, long)]
    input: String,

    /// Output file
    //#[clap(short, long)]
    //output: String,

    /// Make output indexed
    #[clap(long)]
    index: bool,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut filein = BufReader::new(File::open(args.input)?);
    let mut fgb = FgbReader::open(&mut filein)?;
    fgb.process(&mut GeoJsonWriter::new(&mut std::io::stdout()))?;
    Ok(())
}