use clap::{ArgEnum, Args, Parser, Subcommand};
use flatgeobuf::{FallibleStreamingIterator, FgbReader, FgbWriter};
use geozero::error::Result;
use geozero::geojson::{GeoJsonReader, GeoJsonWriter};
use geozero::GeozeroDatasource;
use std::fs::File;
use std::io::{BufRead, BufReader, BufWriter, Write};

#[derive(Parser)]
#[clap(author, version, about, long_about = None)]
#[clap(propagate_version = true)]
struct Cli {
    #[clap(subcommand)]
    command: Commands,
}

#[derive(Subcommand)]
enum Commands {
    /// Convert a file
    Convert(Convert),
    /// Info about a FlatGeobuf file
    Info(Info),
}

#[derive(Args)]
struct Convert {
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

#[derive(Args)]
struct Info {
    /// Input path
    #[clap(short, long)]
    input: Option<String>,

    /// Dump index
    #[clap(long)]
    index: bool,

    /// Dump n-th feature
    #[clap(long)]
    dump: Option<usize>,
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord, ArgEnum)]
enum Format {
    Flatgeobuf,
    Geojson,
}

fn write(
    format: Format,
    reader: impl GeozeroDatasource,
    output: impl Write,
    index: bool,
) -> Result<()> {
    match format {
        Format::Geojson => write_geojson(reader, output)?,
        Format::Flatgeobuf => write_flatgeobuf(reader, output, index)?,
    }
    Ok(())
}

fn write_geojson(mut reader: impl GeozeroDatasource, mut output: impl Write) -> Result<()> {
    let mut writer = GeoJsonWriter::new(&mut output);
    reader.process(&mut writer)?;
    Ok(())
}

fn write_flatgeobuf(
    mut reader: impl GeozeroDatasource,
    mut output: impl Write,
    index: bool,
) -> Result<()> {
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

fn transform(
    inputformat: Format,
    outputformat: Format,
    mut input: impl BufRead,
    output: impl Write,
    index: bool,
) -> Result<()> {
    match inputformat {
        Format::Geojson => write(outputformat, GeoJsonReader(&mut input), output, index)?,
        Format::Flatgeobuf => write(
            outputformat,
            FgbReader::open(&mut input)?.select_all_seq()?,
            output,
            index,
        )?,
    }
    Ok(())
}

fn info(mut input: impl BufRead, args: &Info) -> Result<()> {
    let mut fgb = FgbReader::open(&mut input)?;

    println!("{:#?}", &fgb.header());

    if args.index {
        fgb.process_index(&mut GeoJsonWriter::new(&mut std::io::stdout()))?;
        println!();
    }

    if let Some(fno) = args.dump {
        if args.index {
            println!("Can't dump feature together with index - skipping");
            return Ok(());
        }
        let mut fgb = fgb.select_all_seq()?;
        let mut n = 0;
        while let Some(feature) = fgb.next()? {
            if n == fno {
                println!("{:#?}", &feature.fbs_feature());
                break;
            }
            n += 1;
        }
    }
    Ok(())
}

fn main() -> Result<()> {
    let cli = Cli::parse();
    match &cli.command {
        Commands::Convert(args) => {
            let input: Box<dyn BufRead> = match &args.input {
                Some(x) => Box::new(BufReader::new(File::open(x)?)),
                None => Box::new(BufReader::new(std::io::stdin())),
            };
            let output: Box<dyn Write> = match &args.output {
                Some(x) => Box::new(BufWriter::new(File::create(x)?)),
                None => Box::new(BufWriter::new(std::io::stdout())),
            };
            transform(
                args.inputformat,
                args.outputformat,
                input,
                output,
                args.index,
            )?;
        }
        Commands::Info(args) => {
            let input: Box<dyn BufRead> = match &args.input {
                Some(x) => Box::new(BufReader::new(File::open(x)?)),
                None => Box::new(BufReader::new(std::io::stdin())),
            };
            info(input, args)?;
        }
    }
    Ok(())
}
