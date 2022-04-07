use std::io::BufRead;
use std::io::Write;
use geozero::geojson::GeoJsonReader;
use std::io::BufWriter;
use flatgeobuf::FgbWriter;
use geozero::GeozeroDatasource;
use flatgeobuf::FgbSequentialReader;
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
    input: Option<String>,

    /// Input format
    #[clap(long, arg_enum, default_value_t = Format::Flatgeobuf)]
    inputformat: Format,

    /// Output path
    #[clap(short, long)]
    output: Option<String>,

    /// Output format
    #[clap(long, arg_enum, default_value_t = Format::Flatgeobuf)]
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

fn write(format: Format, reader: impl GeozeroDatasource, output: impl Write, index: bool) -> Result<()> {
    match format {
        Format::Geojson => write_geojson(reader, output)?,
        Format::Flatgeobuf => write_flatgeobuf(reader, output, index)?
    }
    Ok(())
}

fn write_geojson(mut reader: impl GeozeroDatasource, mut output: impl Write) -> Result<()> {
    let mut writer = GeoJsonWriter::new(&mut output);
    reader.process(&mut writer)?;
    Ok(())
}

fn write_flatgeobuf(mut reader: impl GeozeroDatasource, mut output: impl Write, index: bool) -> Result<()> {
    // TODO: would make sense if GeozeroDatasource could provide name and geometry_type?
    let name = "";
    let geometry_type = flatgeobuf::GeometryType::Unknown;
    let mut writer = FgbWriter::create(name, geometry_type, |_builder, header_args| {
        if !index {
            header_args.index_node_size = 0;
        }
    })?;
    reader.process(&mut writer)?;
    writer.write(&mut output)?;
    Ok(())
}

fn transform(inputformat: Format, outputformat: Format, mut input: impl BufRead, output: impl Write, index: bool) -> Result<()> {
    match inputformat {
        Format::Geojson => write(outputformat, GeoJsonReader(&mut input), output, index)?,
        Format::Flatgeobuf => write(outputformat, FgbSequentialReader::open(&mut input)?.select_all()?, output, index)?
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let input: Box<dyn BufRead> = match args.input {
        Some(x) => Box::new(BufReader::new(File::open(x)?)),
        None => Box::new(BufReader::new(std::io::stdin())),
    };
    let output: Box<dyn Write> = match args.output {
        Some(x) => Box::new(BufWriter::new(File::create(x)?)),
        None => Box::new(BufWriter::new(std::io::stdout())),
    };
    transform(args.inputformat, args.outputformat, input, output, args.index)?;
    Ok(())
}
