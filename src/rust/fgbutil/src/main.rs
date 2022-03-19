use geozero::geojson::GeoJsonReader;
use std::io::BufWriter;
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

    /// Input format
    #[clap(long, arg_enum, default_value_t = Format::Flatgeobuf)]
    inputformat: Format,

    /// Output path
    #[clap(short, long)]
    output: String,

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

fn write(format: Format, reader: impl GeozeroDatasource, output: BufWriter<File>) -> Result<()> {
    match format {
        Format::Geojson => write_geojson(reader, output)?,
        Format::Flatgeobuf => write_flatgeobuf(reader, output)?
    }
    Ok(())
}

fn write_geojson(mut reader: impl GeozeroDatasource, mut output: BufWriter<File>) -> Result<()> {
    let mut writer = GeoJsonWriter::new(&mut output);
    reader.process(&mut writer)?;
    Ok(())
}

fn write_flatgeobuf(mut reader: impl GeozeroDatasource, mut output: BufWriter<File>) -> Result<()> {
    // TODO: would make sense if GeozeroDatasource could provide name and geometry_type?
    let name = "";
    let geometry_type = flatgeobuf::GeometryType::Unknown;
    let mut writer = FgbWriter::create(name, geometry_type, |_, _| {})?;
    reader.process(&mut writer)?;
    writer.write(&mut output)?;
    Ok(())
}

fn transform(inputformat: Format, outputformat: Format, mut input: BufReader<File>, output: BufWriter<File>) -> Result<()> {
    match inputformat {
        Format::Geojson => write(outputformat, GeoJsonReader(&mut input), output)?,
        Format::Flatgeobuf => write(outputformat, FgbReader::open(&mut input)?.select_all()?, output)?
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();
    let filein = BufReader::new(File::open(args.input)?);
    let fileout = BufWriter::new(File::create(&args.output)?);
    transform(args.inputformat, args.outputformat, filein, fileout)?;
    Ok(())
}