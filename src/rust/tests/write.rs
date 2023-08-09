use crate::FgbWriter;
use flatgeobuf::*;
use geo_types::{line_string, LineString};
use geozero::error::Result;
use geozero::geojson::{GeoJson, GeoJsonReader};
use geozero::{ColumnValue, GeozeroDatasource, PropertyProcessor};
use std::fs::File;
use std::io::{BufReader, BufWriter, Write};
use tempfile::{tempfile, NamedTempFile};

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
    file.write(buf)?;

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

        file.write(buf)?;
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
    let header = size_prefixed_root_as_header(buf).unwrap();
    assert_eq!(header.features_count(), 1);

    assert!(
        root_as_header(&buf[4..]).is_err(),
        "Verification without size prefix fails"
    );
}

#[test]
fn json_to_fgb() -> Result<()> {
    let mut fgb = FgbWriter::create_with_options(
        "countries",
        GeometryType::MultiPolygon,
        FgbWriterOptions {
            description: Some("Country polygons"),
            crs: FgbCrs {
                code: 4326,
                ..Default::default()
            },
            ..Default::default()
        },
    )?;
    fgb.add_column("fid", ColumnType::ULong, |_fbb, col| {
        col.nullable = false;
    });
    fgb.add_column("name", ColumnType::String, |_, _| {});

    let geojson = GeoJson(
        r#"{"type": "Feature", "properties": {"fid": 42, "name": "New Zealand"}, "geometry": {"type": "MultiPolygon", "coordinates": [[[[173.020375,-40.919052],[173.247234,-41.331999],[173.958405,-40.926701],[174.247587,-41.349155],[174.248517,-41.770008],[173.876447,-42.233184],[173.22274,-42.970038],[172.711246,-43.372288],[173.080113,-43.853344],[172.308584,-43.865694],[171.452925,-44.242519],[171.185138,-44.897104],[170.616697,-45.908929],[169.831422,-46.355775],[169.332331,-46.641235],[168.411354,-46.619945],[167.763745,-46.290197],[166.676886,-46.219917],[166.509144,-45.852705],[167.046424,-45.110941],[168.303763,-44.123973],[168.949409,-43.935819],[169.667815,-43.555326],[170.52492,-43.031688],[171.12509,-42.512754],[171.569714,-41.767424],[171.948709,-41.514417],[172.097227,-40.956104],[172.79858,-40.493962],[173.020375,-40.919052]]],[[[174.612009,-36.156397],[175.336616,-37.209098],[175.357596,-36.526194],[175.808887,-36.798942],[175.95849,-37.555382],[176.763195,-37.881253],[177.438813,-37.961248],[178.010354,-37.579825],[178.517094,-37.695373],[178.274731,-38.582813],[177.97046,-39.166343],[177.206993,-39.145776],[176.939981,-39.449736],[177.032946,-39.879943],[176.885824,-40.065978],[176.508017,-40.604808],[176.01244,-41.289624],[175.239567,-41.688308],[175.067898,-41.425895],[174.650973,-41.281821],[175.22763,-40.459236],[174.900157,-39.908933],[173.824047,-39.508854],[173.852262,-39.146602],[174.574802,-38.797683],[174.743474,-38.027808],[174.697017,-37.381129],[174.292028,-36.711092],[174.319004,-36.534824],[173.840997,-36.121981],[173.054171,-35.237125],[172.636005,-34.529107],[173.007042,-34.450662],[173.551298,-35.006183],[174.32939,-35.265496],[174.612009,-36.156397]]]]}}"#,
    );
    fgb.add_feature(geojson).ok();

    // // Process geometry only and use properties API
    let geom = GeoJson(
        r#"{"type": "MultiPolygon", "coordinates": [[[[31.521001,-29.257387],[31.325561,-29.401978],[30.901763,-29.909957],[30.622813,-30.423776],[30.055716,-31.140269],[28.925553,-32.172041],[28.219756,-32.771953],[27.464608,-33.226964],[26.419452,-33.61495],[25.909664,-33.66704],[25.780628,-33.944646],[25.172862,-33.796851],[24.677853,-33.987176],[23.594043,-33.794474],[22.988189,-33.916431],[22.574157,-33.864083],[21.542799,-34.258839],[20.689053,-34.417175],[20.071261,-34.795137],[19.616405,-34.819166],[19.193278,-34.462599],[18.855315,-34.444306],[18.424643,-33.997873],[18.377411,-34.136521],[18.244499,-33.867752],[18.25008,-33.281431],[17.92519,-32.611291],[18.24791,-32.429131],[18.221762,-31.661633],[17.566918,-30.725721],[17.064416,-29.878641],[17.062918,-29.875954],[16.344977,-28.576705],[16.824017,-28.082162],[17.218929,-28.355943],[17.387497,-28.783514],[17.836152,-28.856378],[18.464899,-29.045462],[19.002127,-28.972443],[19.894734,-28.461105],[19.895768,-24.76779],[20.165726,-24.917962],[20.758609,-25.868136],[20.66647,-26.477453],[20.889609,-26.828543],[21.605896,-26.726534],[22.105969,-26.280256],[22.579532,-25.979448],[22.824271,-25.500459],[23.312097,-25.26869],[23.73357,-25.390129],[24.211267,-25.670216],[25.025171,-25.71967],[25.664666,-25.486816],[25.765849,-25.174845],[25.941652,-24.696373],[26.485753,-24.616327],[26.786407,-24.240691],[27.11941,-23.574323],[28.017236,-22.827754],[29.432188,-22.091313],[29.839037,-22.102216],[30.322883,-22.271612],[30.659865,-22.151567],[31.191409,-22.25151],[31.670398,-23.658969],[31.930589,-24.369417],[31.752408,-25.484284],[31.837778,-25.843332],[31.333158,-25.660191],[31.04408,-25.731452],[30.949667,-26.022649],[30.676609,-26.398078],[30.685962,-26.743845],[31.282773,-27.285879],[31.86806,-27.177927],[32.071665,-26.73382],[32.83012,-26.742192],[32.580265,-27.470158],[32.462133,-28.301011],[32.203389,-28.752405],[31.521001,-29.257387]],[[28.978263,-28.955597],[28.5417,-28.647502],[28.074338,-28.851469],[27.532511,-29.242711],[26.999262,-29.875954],[27.749397,-30.645106],[28.107205,-30.545732],[28.291069,-30.226217],[28.8484,-30.070051],[29.018415,-29.743766],[29.325166,-29.257387],[28.978263,-28.955597]]]]}"#,
    );
    fgb.add_feature_geom(geom, |feat| {
        feat.property(0, "fid", &ColumnValue::Long(43)).unwrap();
        feat.property(1, "name", &ColumnValue::String("South Africa"))
            .unwrap();
    })
    .ok();

    // let mut file = BufWriter::new(File::create("test_multipoly.fgb")?);
    let mut file = BufWriter::new(tempfile()?);
    fgb.write(&mut file)?;

    Ok(())
}

#[test]
fn geozero_to_fgb() -> Result<()> {
    let mut fgb = FgbWriter::create("countries", GeometryType::MultiPolygon)?;
    let mut fin = BufReader::new(File::open("../../test/data/countries.geojson")?);
    let mut reader = GeoJsonReader(&mut fin);
    reader.process(&mut fgb)?;
    // let mut fout = BufWriter::new(File::create("test_multipoly.fgb")?);
    let mut fout = BufWriter::new(tempfile()?);
    fgb.write(&mut fout)?;

    Ok(())
}

#[test]
fn test_save_fgb_and_load() -> Result<()> {
    let file_to_write = NamedTempFile::new()?;

    // Save
    let linestrings: Vec<LineString<f64>> = vec![
        geo_types::line_string![
            (x: -21.95156, y: 64.1446),
            (x: -21.951, y: 64.14479),
            (x: -21.95044, y: 64.14527),
            (x: -21.951445, y: 64.145508)
        ],
        geo_types::line_string![(x: 0.0, y: 0.0), (x: 1.0, y: 1.0),],
    ];

    let mut fgb = FgbWriter::create_with_options(
        "test_write",
        GeometryType::LineString,
        FgbWriterOptions {
            write_index: false,
            crs: FgbCrs {
                code: 4326,
                ..Default::default()
            },
            ..Default::default()
        },
    )?;

    for geom in linestrings.iter() {
        let geom: geo_types::Geometry<f64> = geom.to_owned().into();
        fgb.add_feature_geom(geom, |_feat| {})?;
    }
    let mut file = BufWriter::new(&file_to_write);
    fgb.write(&mut file)?;

    file.flush()?;

    // Load
    let read_file_again = file_to_write.reopen()?;
    let mut filein = BufReader::new(&read_file_again);
    let mut fgb = FgbReader::open(&mut filein)?.select_all()?;
    let mut cnt = 0;
    while let Some(feature) = fgb.next().unwrap() {
        let _props = feature.properties();
        let _geometry = feature.geometry().unwrap();
        cnt += 1
    }
    assert_eq!(cnt, 2);

    Ok(())
}
