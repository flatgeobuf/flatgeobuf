use criterion::{criterion_group, criterion_main, Criterion};
use flatgeobuf::*;
use geozero::error::Result;
use geozero::ProcessorSink;
use seek_bufread::BufReader;
use std::fs::File;

fn read_fgb() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
    fgb.process_features(&mut ProcessorSink)?;
    Ok(())
}

fn read_fgb_process_geom() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;

    let mut null_reader = ProcessorSink;
    while let Some(feature) = fgb.next()? {
        feature.process_geom(&mut null_reader)?;
    }

    Ok(())
}

fn read_fgb_unchecked() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = unsafe { FgbReader::open_unchecked(&mut filein) }?.select_all()?;
    fgb.process_features(&mut ProcessorSink)?;
    Ok(())
}

fn read_fgb_seq() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all_seq()?;
    fgb.process_features(&mut ProcessorSink)?;
    Ok(())
}

fn read_bbox() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    fgb.process_features(&mut ProcessorSink)?;
    Ok(())
}

fn read_bbox_seq() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_bbox_seq(8.8, 47.2, 9.5, 55.3)?;
    fgb.process_features(&mut ProcessorSink)?;
    Ok(())
}

// fn read_header(fname: &str) -> Result<(File, FgbReader)> {
//     let fin = File::open(fname)?;
//     let mut filein = BufReader::new(fin);
//     let fgb = FgbReader::open(&mut filein)?;
//     Ok((fin, fgb))
// }

// fn select_bbox(fgb: FgbReader) -> Result<()> {
//     let _count = fgb.select_bbox(8.8, 47.2, 9.5, 55.3)?;
//     Ok(())
// }

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read_fgb", |b| b.iter(|| read_fgb()));
    c.bench_function("read_fgb_process_geom", |b| {
        b.iter(|| read_fgb_process_geom())
    });
    c.bench_function("read_fgb_unchecked", |b| b.iter(|| read_fgb_unchecked()));
    c.bench_function("read_fgb_seq", |b| b.iter(|| read_fgb_seq()));
    c.bench_function("read_bbox", |b| b.iter(|| read_bbox()));
    c.bench_function("read_bbox_seq", |b| b.iter(|| read_bbox_seq()));
    // c.bench_function("select_bbox", move |b| {
    //     b.iter_with_setup(
    //         || read_header("../../test/data/countries.fgb").unwrap(),
    //         |(mut filein, fgb)| select_bbox(fgb),
    //     )
    // });
    // c.bench_function("select_bbox_big_index", move |b| {
    //     b.iter_with_setup(
    //         || read_header("../../test/data/osm/osm-buildings-ch.fgb").unwrap(),
    //         // 2'396'905 features (8.522086, 47.363333, 8.553521, 47.376020)
    //         |(mut filein, hreader)| select_bbox(&mut filein, hreader.header()),
    //     )
    // });
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10); targets=criterion_benchmark);
criterion_main!(benches);
