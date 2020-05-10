use criterion::{criterion_group, criterion_main, Criterion};
use flatgeobuf::*;
use geozero::error::Result;
use geozero::GeomProcessor;
use std::fs::File;
use std::io::BufReader;

struct NullReader;
impl GeomProcessor for NullReader {}

fn read_fgb() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    let geometry_type = fgb.header().geometry_type();
    fgb.select_all()?;

    let mut null_reader = NullReader;
    while let Some(feature) = fgb.next()? {
        let geometry = feature.geometry().unwrap();
        geometry.process(&mut null_reader, geometry_type)?;
    }

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
