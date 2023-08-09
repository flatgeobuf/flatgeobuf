use criterion::{criterion_group, criterion_main, Criterion};
use flatgeobuf::*;
use geozero::error::Result;
use geozero::geojson::GeoJsonWriter;
use seek_bufread::BufReader;
use std::fs::File;
use std::io::BufWriter;
use tempfile::tempfile;

fn fgb_to_geojson() -> Result<()> {
    // Comparison: time ogr2ogr -f GeoJSON /tmp/countries-ogr.json ../../test/data/countries.fgb
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
    let mut fout = BufWriter::new(tempfile()?); // or File::create("/tmp/countries.json")
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json)
}

fn fgb_to_geojson_unchecked() -> Result<()> {
    // Comparison: time ogr2ogr -f GeoJSON -oo VERIFY_BUFFERS=NO /tmp/countries-ogr.json ../../test/data/countries.fgb
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = unsafe { FgbReader::open_unchecked(&mut filein) }?.select_all()?;
    let mut fout = BufWriter::new(tempfile()?); // or File::create("/tmp/countries.json")
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json)
}

fn fgb_to_geojson_dev_null() -> Result<()> {
    // Comparison: time ogr2ogr -f GeoJSON /dev/null ../../test/data/countries.fgb
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
    let mut fout = std::io::sink();
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json)
}

fn fgb_to_geojson_dev_null_unchecked() -> Result<()> {
    // Comparison: time ogr2ogr -f GeoJSON -oo VERIFY_BUFFERS=NO /dev/null ../../test/data/countries.fgb
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = unsafe { FgbReader::open_unchecked(&mut filein) }?.select_all()?;
    let mut fout = std::io::sink();
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json)
}

fn fgb_bbox_to_geojson_dev_null() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    let mut fout = std::io::sink();
    let mut json = GeoJsonWriter::new(&mut fout);
    fgb.process_features(&mut json)
}

fn criterion_benchmark(c: &mut Criterion) {
    c.bench_function("fgb_to_geojson", |b| b.iter(fgb_to_geojson));
    c.bench_function("fgb_to_geojson_unchecked", |b| {
        b.iter(fgb_to_geojson_unchecked)
    });
    c.bench_function("fgb_to_geojson_dev_null", |b| {
        b.iter(fgb_to_geojson_dev_null)
    });
    c.bench_function("fgb_to_geojson_dev_null_unchecked", |b| {
        b.iter(fgb_to_geojson_dev_null_unchecked)
    });
    c.bench_function("fgb_bbox_to_geojson_dev_null", |b| {
        b.iter(fgb_bbox_to_geojson_dev_null)
    });
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10); targets=criterion_benchmark);
criterion_main!(benches);
