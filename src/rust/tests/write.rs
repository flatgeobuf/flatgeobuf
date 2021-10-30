use crate::FeatureWriter;
use flatgeobuf::*;
use geozero::geojson::GeoJson;
use geozero::GeozeroDatasource;
use std::io::{BufWriter, Write};
use tempfile::tempfile;

#[test]
fn write_file() -> std::io::Result<()> {
    let mut file = BufWriter::new(tempfile()?);
    let points = [[1.0, 1.0], [2.0, 2.0]];

    const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', 3, b'f', b'g', b'b', 0];
    file.write(&MAGIC_BYTES)?;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let column_args = ColumnArgs {
        name: Some(fbb.create_string("STATE_FIPS")),
        type_: ColumnType::String,
        ..Default::default()
    };
    let column = Column::create(&mut fbb, &column_args);
    let header_args = HeaderArgs {
        name: Some(fbb.create_string("Test1")),
        geometry_type: GeometryType::Point,
        columns: Some(fbb.create_vector(&[column])),
        features_count: points.len() as u64,
        index_node_size: 0,
        ..Default::default()
    };

    let header = Header::create(&mut fbb, &header_args);
    fbb.finish_size_prefixed(header, None);
    let buf = fbb.finished_data();
    file.write(&buf)?;

    for point in points {
        let mut fbb = flatbuffers::FlatBufferBuilder::new();
        let xy = fbb.create_vector(&point);
        let g = Geometry::create(
            &mut fbb,
            &GeometryArgs {
                xy: Some(xy),
                ..Default::default()
            },
        );
        let f = Feature::create(
            &mut fbb,
            &FeatureArgs {
                geometry: Some(g),
                ..Default::default()
            },
        );
        fbb.finish_size_prefixed(f, None);
        let buf = fbb.finished_data();
        assert_eq!(buf.len(), 64);

        file.write(&buf)?;
    }

    Ok(())
}

#[test]
fn verify_header() {
    let mut builder = flatbuffers::FlatBufferBuilder::with_capacity(1024);
    builder.start_vector::<u8>(0);
    let empty_vec = builder.end_vector(0);
    let header_args = HeaderArgs {
        name: Some(builder.create_string("triangle")),
        envelope: Some(builder.create_vector(&[0.0, 0.0, 9.0, 9.0])),
        geometry_type: GeometryType::Triangle,
        columns: Some(empty_vec),
        features_count: 1,
        ..Default::default()
    };
    let header = Header::create(&mut builder, &header_args);
    builder.finish_size_prefixed(header, None);
    let buf = builder.finished_data();

    // verify
    let header = size_prefixed_root_as_header(&buf).unwrap();
    assert_eq!(header.features_count(), 1);

    assert!(
        root_as_header(&buf[4..]).is_err(),
        "Verfication without size prefix fails"
    );
}

#[test]
fn json_to_fgb() -> std::io::Result<()> {
    // let mut file = BufWriter::new(File::create("test_multipoly.fgb")?);
    let mut file = BufWriter::new(tempfile()?);

    const MAGIC_BYTES: [u8; 8] = [b'f', b'g', b'b', 3, b'f', b'g', b'b', 0];
    file.write(&MAGIC_BYTES)?;

    let mut fbb = flatbuffers::FlatBufferBuilder::new();
    let col0 = ColumnArgs {
        name: Some(fbb.create_string("fid")),
        type_: ColumnType::Int,
        ..Default::default()
    };
    let col0 = Column::create(&mut fbb, &col0);
    let col1 = ColumnArgs {
        name: Some(fbb.create_string("name")),
        type_: ColumnType::String,
        ..Default::default()
    };
    let col1 = Column::create(&mut fbb, &col1);
    let header_args = HeaderArgs {
        name: Some(fbb.create_string("countries")),
        geometry_type: GeometryType::MultiPolygon,
        columns: Some(fbb.create_vector(&[col0, col1])),
        features_count: 1,
        index_node_size: 0,
        ..Default::default()
    };

    let header = Header::create(&mut fbb, &header_args);
    fbb.finish_size_prefixed(header, None);
    let buf = fbb.finished_data();
    file.write(&buf)?;

    let mut geojson = GeoJson(
        r#"{"type": "Feature", "properties": {"fid": 42, "name": "New Zealand"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}"#,
    );
    let mut fgb_writer = FeatureWriter::new();
    assert!(geojson.process(&mut fgb_writer).is_ok());
    let feat = fgb_writer.to_feature(buf.to_vec());
    feat.write(&mut file)?;

    Ok(())
}
