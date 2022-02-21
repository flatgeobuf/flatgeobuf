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


/*fn write(format: Format, reader: Box<GeozeroDatasource>)
{

}*/

fn main() -> Result<()> {
    let args = Args::parse();

    let mut filein = BufReader::new(File::open(args.input)?);
    let mut fileout = BufWriter::new(File::create(&args.output)?);

    match args.inputformat {
        Format::Geojson => {},
        Format::Flatgeobuf => {
            let mut reader = Box::new(FgbReader::open(&mut filein)?);
            reader.select_all()?;
            let name = reader.header().name().unwrap();
            let geometry_type = reader.header().geometry_type();
            //write(args.outputformat, reader);
            match args.outputformat {
                Format::Geojson => {
                    let mut writer = GeoJsonWriter::new(&mut fileout);
                    reader.process(&mut writer)?;
                },
                Format::Flatgeobuf => {
                    let mut writer = FgbWriter::create(name, geometry_type, |_, _| {})?;
                    reader.process(&mut writer)?;
                    writer.write(&mut fileout)?;
                }
            }
        }
    }

    Ok(())
}