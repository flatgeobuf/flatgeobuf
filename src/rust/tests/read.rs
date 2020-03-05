use flatgeobuf::*;
use std::error::Error;
use std::io::{BufReader, Read, Seek, SeekFrom};

#[test]
fn read_file() -> std::result::Result<(), std::io::Error> {
    let f = std::fs::File::open("../../test/data/countries.fgb")?;
    let mut reader = BufReader::new(f);

    let mut magic_buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut magic_buf)?;
    assert_eq!(magic_buf, MAGIC_BYTES);

    let mut size_buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut size_buf)?;
    let header_size = u32::from_le_bytes(size_buf);
    assert_eq!(header_size, 604);

    let mut header_buf = vec![0; header_size as usize];
    reader.read_exact(&mut header_buf)?;

    let header = get_root_as_header(&header_buf[..]);
    assert_eq!(header.name(), Some("countries"));
    assert!(header.envelope().is_some());
    assert_eq!(
        header.envelope().unwrap().safe_slice(),
        &[-180.0, -85.609038, 180.0, 83.64513]
    );
    assert_eq!(header.geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(header.hasZ(), false);
    assert_eq!(header.hasM(), false);
    assert_eq!(header.hasT(), false);
    assert_eq!(header.hasTM(), false);
    assert!(header.columns().is_some());
    let columns = header.columns().unwrap();
    assert_eq!(columns.len(), 2);
    let column0 = columns.get(0);
    assert_eq!(column0.name(), "id");
    assert_eq!(column0.type_(), ColumnType::String);
    assert_eq!(header.features_count(), 179);
    assert_eq!(header.index_node_size(), 16);
    assert!(header.crs().is_some());
    let crs = header.crs().unwrap();
    assert_eq!(crs.code(), 4326);

    // Skip index
    let index_size = packed_rtree_size(header.features_count(), header.index_node_size());
    reader.seek(SeekFrom::Current(index_size as i64))?;

    // Read first feature
    reader.read_exact(&mut size_buf)?;
    let feature_size = u32::from_le_bytes(size_buf);
    assert_eq!(feature_size, 10804);
    let mut feature_buf = vec![0; feature_size as usize];
    reader.read_exact(&mut feature_buf)?;

    let feature = get_root_as_feature(&feature_buf[..]);
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::MultiPolygon);

    let parts = geometry.parts().unwrap();
    let mut num_vertices = 0;
    for i in 0..parts.len() {
        let part = parts.get(i);
        for _j in 0..part.xy().unwrap().len() {
            num_vertices += 1;
        }
    }
    assert_eq!(num_vertices, 1316);

    assert!(feature.properties().is_some());
    assert!(feature.columns().is_none());
    Ok(())
}

#[test]
fn file_reader() -> std::result::Result<(), std::io::Error> {
    let f = std::fs::File::open("../../test/data/countries.fgb")?;
    let mut reader = Reader::new(f);
    let header = reader.read_header()?;
    let cnt = header.features_count();
    assert_eq!(cnt, 179);
    reader.select_all()?;
    let mut num_features = 0;
    while let Ok(_feature) = reader.next() {
        num_features += 1;
    }
    assert_eq!(cnt, num_features);

    let f = std::fs::File::open("../../test/data/states.geojson")?;
    let mut reader = Reader::new(f);
    assert_eq!(
        reader.read_header().err().unwrap().description(),
        "Magic byte doesn\'t match"
    );

    Ok(())
}
