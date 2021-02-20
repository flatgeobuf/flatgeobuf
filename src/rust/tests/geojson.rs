use flatgeobuf::*;
use geozero::error::Result;
use geozero_core::geojson::GeoJsonWriter;
use std::fs::File;
use std::io::BufReader;

#[test]
fn fgb_to_geojson() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    fgb.select_all()?;
    let mut json_data: Vec<u8> = Vec::new();
    let mut json = GeoJsonWriter::new(&mut json_data);
    fgb.process_features(&mut json)?;
    assert_eq!(
        &std::str::from_utf8(&json_data).unwrap()[0..215],
        r#"{
"type": "FeatureCollection",
"name": "countries",
"features": [{"type": "Feature", "properties": {"id": "ATA", "name": "Antarctica"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[-59.572095,-80.040179],"#
    );
    Ok(())
}

#[test]
#[ignore]
fn num_properties() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/ne_10m_geographic_lines.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    let mut out: Vec<u8> = Vec::new();
    let mut json = GeoJsonWriter::new(&mut out);
    feature.process(&mut json, 0)?;
    assert_eq!(
        &std::str::from_utf8(&out).unwrap()[..293],
        r#"{"type": "Feature", "properties": {"scalerank": 2, "name": "Tropic of Cancer", "name_long": "Tropic of Cancer", "abbrev": "Tr. of Cancer", "note": "Northern tropic, 23.4° N.", "featurecla": "Circle of latitude", "min_zoom": 4.1, "wikidataid": "Q176635", "name_ar": "مدار السرطان", "#
    );

    Ok(())
}

#[cfg(feature = "http")]
async fn http_json_async() -> Result<()> {
    let url = "https://github.com/flatgeobuf/flatgeobuf/raw/master/test/data/countries.fgb";
    let mut fgb = HttpFgbReader::open(url).await?;
    fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;

    let mut json_data: Vec<u8> = Vec::new();
    let mut json = GeoJsonWriter::new(&mut json_data);
    fgb.process_features(&mut json).await?;
    assert_eq!(
        &std::str::from_utf8(&json_data).unwrap()[..239],
        r#"{
"type": "FeatureCollection",
"name": "countries",
"features": [{"type": "Feature", "properties": {"id": "DNK", "name": "Denmark"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[12.690006,55.609991],[12.089991,54.800015],[11.043"#
    );
    Ok(())
}

#[test]
#[cfg(feature = "http")]
fn http_json() {
    assert!(tokio::runtime::Runtime::new()
        .unwrap()
        .block_on(http_json_async())
        .is_ok());
}
