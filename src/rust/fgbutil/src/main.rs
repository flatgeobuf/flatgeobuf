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
    // Input path
    #[clap(short, long)]
    input: String,

    // Input format
    #[clap(long, arg_enum, default_value_t = Format::Flatgeobuf)]
    inputformat: Format,

    // Output path
    #[clap(short, long)]
    output: String,

    // Output format
    #[clap(long, arg_enum, default_value_t = Format::Flatgeobuf)]
    outputformat: Format,

    // Make output indexed
    #[clap(long)]
    index: bool,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Format {
    Flatgeobuf,
    Geojson,
}

// TODO: Cannot pass around GeozeroDatasource because it is a non object safe trait?
// TODO: Regardless might need dyn / Box?

fn write(format: Format, reader: GeozeroDatasource, output: BufWriter<File>) -> Result<()> {
    match format {
        Format::Geojson => write_flatgeobuf(reader, output)?,
        Format::Flatgeobuf => write_flatgeobuf(reader, output)?
    }
    Ok(())
}

fn write_geojson(reader: GeozeroDatasource, output: BufWriter<File>) -> Result<()> {
    let mut writer = GeoJsonWriter::new(&mut output);
    reader.process(&mut writer)?;
    Ok(())
}

fn write_flatgeobuf(reader: GeozeroDatasource, output: BufWriter<File>) -> Result<()> {
    // TODO: would make sense if GeozeroDatasource could provide name and geometry_type?
    let name = "";
    let geometry_type = flatgeobuf::GeometryType::Unknown;
    let mut writer = FgbWriter::create(name, geometry_type, |_, _| {})?;
    reader.process(&mut writer)?;
    writer.write(&mut output)?;
    Ok(())
}

fn transform(inputformat: Format, outputformat: Format, input: BufReader<File>, output: BufWriter<File>) -> Result<()> {
    match inputformat {
        Format::Geojson => {
            // TODO: impl..
        },
        Format::Flatgeobuf => {
            let mut reader = FgbReader::open(&mut input)?;
            reader.select_all()?;
            write(outputformat, reader, output);
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let args = Args::parse();

    let mut filein = BufReader::new(File::open(args.input)?);
    let mut fileout = BufWriter::new(File::create(&args.output)?);

    transform(args.inputformat, args.outputformat, filein, fileout);

    Ok(())
}