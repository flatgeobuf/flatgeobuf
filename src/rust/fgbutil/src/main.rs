use geozero::GeozeroDatasource;
use flatgeobuf::FgbReader;
use std::fs::File;
use std::io::BufReader;
use geozero::error::{Result};
use geozero::geojson::GeoJsonWriter;
use clap::{ArgEnum, Parser};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
struct Args {
    /// Input path
    #[clap(short, long)]
    input: String,

    #[clap(long, arg_enum, default_value_t = Format::Flatgeobuf)]
    inputformat: Format,

    /// Output path
    //#[clap(short, long)]
    //output: String,

    #[clap(long, arg_enum, default_value_t = Format::Geojson)]
    outputformat: Format,

    /// Make output indexed
    #[clap(long)]
    index: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Format {
    Flatgeobuf,
    Geojson,
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut filein = BufReader::new(File::open(args.input)?);
    let mut reader = FgbReader::open(&mut filein)?.select_all()?;
    let mut output = std::io::stdout();
    let mut writer = GeoJsonWriter::new(&mut output);
    reader.process(&mut writer)?;
    Ok(())
}