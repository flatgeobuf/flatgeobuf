use std::io::Stdout;
use std::io::Write;
use flatgeobuf::FgbWriter;
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

enum FormatProcessor<'a, W: Write> {
    Geojson(GeoJsonWriter<'a, W>),
    Flatgeobuf(FgbWriter<'a>),
}

fn main() -> Result<()> {
    let args = Args::parse();
    let mut filein = BufReader::new(File::open(args.input)?);
    let mut reader = FgbReader::open(&mut filein)?;
    let name = reader.header().name().unwrap();
    let geometry_type = reader.header().geometry_type();
    let mut output = std::io::stdout();
    let processor: FormatProcessor<Stdout> = match args.inputformat {
        Format::Geojson => FormatProcessor::Geojson(GeoJsonWriter::new(&mut output)),
        Format::Flatgeobuf => FormatProcessor::Flatgeobuf(FgbWriter::create(name, geometry_type, |_, _| {})?),
    };
    reader.select_all()?;
    match processor {
        FormatProcessor::Geojson(mut value) => reader.process(&mut value)?,
        FormatProcessor::Flatgeobuf(mut value) => reader.process(&mut value)?
    }
    Ok(())
}