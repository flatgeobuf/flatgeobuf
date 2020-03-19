use criterion::{criterion_group, criterion_main, Criterion};
use flatgeobuf::*;
use std::fs::File;
use std::io::{BufReader, BufWriter};
use tempfile::tempfile;

struct NullReader;
impl GeomReader for NullReader {}

fn read_fgb() -> std::result::Result<(), std::io::Error> {
    let fin = File::open("../../test/data/countries.fgb")?;
    let mut filein = BufReader::new(fin);
    let hreader = HeaderReader::read(&mut filein)?;
    let header = hreader.header();

    let mut freader = FeatureReader::select_all(&mut filein, &header)?;

    let mut null_reader = NullReader;
    while let Ok(feature) = freader.next(&mut filein) {
        let geometry = feature.geometry().unwrap();
        geometry.parse(&mut null_reader, header.geometry_type());
    }

    Ok(())
}

fn fgb_to_geojson() -> std::result::Result<(), std::io::Error> {
    // Comparison: time ogr2ogr -f GeoJSON -oo VERIFY_BUFFERS=NO /tmp/countries-ogr.json ../../test/data/countries.fgb
    let fin = File::open("../../test/data/countries.fgb")?;
    let mut filein = BufReader::new(fin);
    let hreader = HeaderReader::read(&mut filein)?;
    let header = hreader.header();

    let mut freader = FeatureReader::select_all(&mut filein, &header)?;

    let mut fout = BufWriter::new(tempfile()?); // or File::create("/tmp/countries.json")
    freader.to_geojson(&mut filein, &header, &mut fout)
}

fn fgb_to_geojson_dev_null() -> std::result::Result<(), std::io::Error> {
    // Comparison: time ogr2ogr -f GeoJSON -oo VERIFY_BUFFERS=NO /dev/null ../../test/data/countries.fgb
    let fin = File::open("../../test/data/countries.fgb")?;
    let mut filein = BufReader::new(fin);
    let hreader = HeaderReader::read(&mut filein)?;
    let header = hreader.header();

    let mut freader = FeatureReader::select_all(&mut filein, &header)?;

    let mut fout = std::io::sink();
    freader.to_geojson(&mut filein, &header, &mut fout)
}

fn read_header() -> std::result::Result<(BufReader<File>, HeaderReader), std::io::Error> {
    let fin = File::open("../../test/data/countries.fgb")?;
    let mut filein = BufReader::new(fin);
    let hreader = HeaderReader::read(&mut filein)?;
    Ok((filein, hreader))
}

fn select_bbox(
    mut filein: &mut BufReader<File>,
    header: Header,
) -> std::result::Result<(), std::io::Error> {
    let _freader = FeatureReader::select_bbox(&mut filein, &header, 8.8, 47.2, 9.5, 55.3)?;
    Ok(())
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("read_fgb", |b| b.iter(|| read_fgb()));
    c.bench_function("fgb_to_geojson", |b| b.iter(|| fgb_to_geojson()));
    c.bench_function("fgb_to_geojson_dev_null", |b| b.iter(|| fgb_to_geojson_dev_null()));
    c.bench_function("select_bbox", move |b| {
        b.iter_with_setup(
            || read_header().unwrap(),
            |(mut filein, hreader)| select_bbox(&mut filein, hreader.header()),
        )
    });
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10); targets=criterion_benchmark);
criterion_main!(benches);
