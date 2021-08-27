use flatgeobuf::*;
use geozero::error::Result;
use geozero::wkt::WktWriter;
use geozero::{ColumnValue, CoordDimensions, GeomProcessor, PropertyProcessor, ToWkt};
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

#[test]
fn reader_headers_checked() {
    assert!(
        FgbReader::open(&mut File::open("../../test/data/surface/triangle.fgb").unwrap()).is_ok()
    );
    assert!(FgbReader::open(&mut File::open("../../test/data/topp_states.fgb").unwrap()).is_ok());
    assert!(FgbReader::open(&mut File::open("../../test/data/UScounties.fgb").unwrap()).is_ok());
    assert!(FgbReader::open(&mut File::open("../../test/data/countries.fgb").unwrap()).is_ok());
}

#[test]
fn read_file_low_level() -> Result<()> {
    let f = File::open("../../test/data/countries.fgb")?;
    let mut reader = BufReader::new(f);

    let mut magic_buf: [u8; 8] = [0; 8];
    reader.read_exact(&mut magic_buf)?;
    assert_eq!(magic_buf, [b'f', b'g', b'b', VERSION, b'f', b'g', b'b', 0]);

    let mut size_buf: [u8; 4] = [0; 4];
    reader.read_exact(&mut size_buf)?;
    let header_size = u32::from_le_bytes(size_buf) as usize;
    assert_eq!(header_size, 604);
    let mut header_buf = Vec::with_capacity(header_size + 4);
    header_buf.extend_from_slice(&size_buf);
    header_buf.resize(header_buf.capacity(), 0);
    reader.read_exact(&mut header_buf[4..])?;

    let header = size_prefixed_root_as_header(&header_buf).unwrap();
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
    let index_size =
        PackedRTree::index_size(header.features_count() as usize, header.index_node_size());
    reader.seek(SeekFrom::Current(index_size as i64))?;

    // Read first feature
    reader.read_exact(&mut size_buf)?;
    let feature_size = u32::from_le_bytes(size_buf) as usize;
    assert_eq!(feature_size, 10804);
    let mut feature_buf = Vec::with_capacity(feature_size + 4);
    feature_buf.extend_from_slice(&size_buf);
    feature_buf.resize(feature_buf.capacity(), 0);
    reader.read_exact(&mut feature_buf[4..])?;

    let feature = size_prefixed_root_as_feature(&feature_buf).unwrap();
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::MultiPolygon);

    let parts = geometry.parts().unwrap();
    let mut num_vertices = 0;
    for i in 0..parts.len() {
        let part = parts.get(i);
        for _j in (0..part.xy().unwrap().len()).step_by(2) {
            num_vertices += 1;
        }
    }
    assert_eq!(num_vertices, 658);

    assert!(feature.properties().is_some());
    assert!(feature.columns().is_none());
    Ok(())
}

#[test]
fn read_all() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    fgb.select_all()?;
    let mut cnt = 0;
    while let Some(feature) = fgb.next()? {
        let _props = feature.properties()?;
        let _geometry = feature.geometry().unwrap();
        cnt += 1
    }
    assert_eq!(cnt, 179);
    Ok(())
}

struct VertexCounter(u64);

impl GeomProcessor for VertexCounter {
    fn xy(&mut self, _x: f64, _y: f64, _idx: usize) -> Result<()> {
        self.0 += 1;
        Ok(())
    }
}

#[test]
fn filter() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    fgb.select_all()?;
    let mut cnt = 0;
    while let Some(feature) = fgb
        .by_ref()
        .filter(|feat| feat.property("id") == Some("DNK".to_string()))
        .next()?
    {
        let _geometry = feature.geometry().unwrap();
        cnt += 1
    }
    assert_eq!(cnt, 1);
    Ok(())
}

#[test]
fn file_reader() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(fgb.header().features_count(), 179);

    let count = fgb.select_all()?;
    assert_eq!(count, 179);

    if let Some(feature) = fgb.find(|feat| feat.property_n(0) == Some("DNK".to_string()))? {
        // OGRFeature(countries):46
        //   id (String) = DNK
        //   name (String) = Denmark
        //   MULTIPOLYGON (((12.690006 55.609991,12.089991 54.800015,11.043543 55.364864,10.903914 55.779955,12.370904 56.111407,12.690006 55.609991)),((10.912182 56.458621,1
        // 0.667804 56.081383,10.369993 56.190007,9.649985 55.469999,9.921906 54.983104,9.282049 54.830865,8.526229 54.962744,8.120311 55.517723,8.089977 56.540012,8.256582 5
        // 6.809969,8.543438 57.110003,9.424469 57.172066,9.775559 57.447941,10.580006 57.730017,10.546106 57.215733,10.25 56.890016,10.369993 56.609982,10.912182 56.458621))
        // )

        let mut vertex_counter = VertexCounter(0);
        feature.process_geom(&mut vertex_counter)?;
        assert_eq!(vertex_counter.0, 24);

        let props = feature.properties()?;
        assert_eq!(&props["id"], "DNK");
        assert_eq!(&props["name"], "Denmark");
    } else {
        assert!(false, "find failed");
    }

    Ok(())
}

#[test]
fn bbox_file_reader() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/countries.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    let count = fgb.select_bbox(8.8, 47.2, 9.5, 55.3)?;
    assert_eq!(count, 6);

    let feature = fgb.next()?.unwrap();
    assert_eq!(feature.property("name"), Some("Denmark".to_string()));

    Ok(())
}

#[test]
fn magic_byte() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/states.geojson")?);
    assert_eq!(
        FgbReader::open(&mut filein).err().unwrap().to_string(),
        "geometry format"
    );

    Ok(())
}

#[test]
#[ignore]
fn point_layer() -> Result<()> {
    let mut filein = BufReader::new(File::open(
        "../../test/data/ne_10m_admin_0_country_points.fgb",
    )?);
    let mut fgb = FgbReader::open(&mut filein)?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::Point);
    assert_eq!(fgb.header().features_count(), 250);

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::Unknown);
    let xy = geometry.xy().unwrap();
    assert_eq!(
        (xy.get(0), xy.get(1)),
        (2223639.4731508396, -15878634.348995442)
    );
    let _props = feature.properties()?;

    Ok(())
}

#[test]
#[ignore]
fn linestring_layer() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/lines.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::LineString);
    assert_eq!(fgb.header().features_count(), 8375);

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::Unknown);
    let xy = geometry.xy().unwrap();
    let mut line = Vec::with_capacity(xy.len() / 2);
    for i in (0..xy.len()).step_by(2) {
        line.push((xy.get(i), xy.get(i + 1)));
    }
    assert_eq!(line.len(), 7);
    assert_eq!(line[0], (1875038.4476102313, -3269648.6879248763));

    assert_eq!(feature.to_wkt().unwrap(), "LINESTRING(1875038.4476102313 -3269648.6879248763,1874359.6415041967 -3270196.8129848638,1874141.0428635243 -3270953.7840121365,1874440.1778162003 -3271619.4315206874,1876396.0598222911 -3274138.747656357,1876442.0805243007 -3275052.60551469,1874739.312657555 -3275457.333765534)");

    let _props = feature.properties()?;

    Ok(())
}

#[test]
#[ignore]
fn geomcollection_layer() -> Result<()> {
    let mut filein = BufReader::new(File::open(
        "../../test/data/gdal_sample_v1.2_nonlinear/geomcollection2d.fgb",
    )?);
    let mut fgb = FgbReader::open(&mut filein)?;
    assert_eq!(
        fgb.header().geometry_type(),
        GeometryType::GeometryCollection
    );
    assert_eq!(fgb.header().features_count(), 1);

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    assert!(feature.geometry().is_some());
    let wkt = feature.to_wkt()?;
    assert_eq!(
        &wkt,
        "GEOMETRYCOLLECTION(POINT(0 1),LINESTRING(2 3,4 5),POLYGON((0 0,0 10,10 10,10 0,0 0),(1 1,1 9,9 9,9 1,1 1)),MULTIPOINT(0 1,2 3),MULTILINESTRING((0 1,2 3),(4 5,6 7)),MULTIPOLYGON(((0 0,0 10,10 10,10 0,0 0),(1 1,1 9,9 9,9 1,1 1)),((-9 0,-9 10,-1 10,-1 0,-9 0))))"
    );

    let _props = feature.properties()?;

    Ok(())
}

fn read_layer_geometry(fname: &str, with_z: bool) -> Result<String> {
    let mut filein = BufReader::new(File::open(&format!("../../test/data/{}", fname))?);
    let mut fgb = FgbReader::open(&mut filein)?;

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    assert!(feature.geometry().is_some());

    let mut wkt_data: Vec<u8> = Vec::new();
    let mut processor = WktWriter::new(&mut wkt_data);
    processor.dims.z = with_z;
    feature.process_geom(&mut processor)?;
    Ok(std::str::from_utf8(&wkt_data).unwrap().to_string())
}

#[test]
#[ignore]
fn curve_layers() -> Result<()> {
    assert_eq!(
        &read_layer_geometry("gdal_sample_v1.2_nonlinear/circularstring.fgb", false)?,
        "CIRCULARSTRING(0 0,1 1,2 0)"
    );

    assert_eq!(
        &read_layer_geometry("gdal_sample_v1.2_nonlinear/compoundcurve.fgb", false)?,
        "COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0))"
    );

    assert_eq!(
        &read_layer_geometry("gdal_sample_v1.2_nonlinear/multicurve.fgb", false)?,
        "MULTICURVE(CIRCULARSTRING(0 0,1 1,2 0))"
    );

    assert_eq!(
        &read_layer_geometry("gdal_sample_v1.2_nonlinear/curvepolygon.fgb", false)?,
        "CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0)))"
    );

    assert_eq!(
        &read_layer_geometry("gdal_sample_v1.2_nonlinear/multisurface.fgb", false)?,
        "MULTISURFACE(CURVEPOLYGON(COMPOUNDCURVE(CIRCULARSTRING(0 0,1 1,2 0),(2 0,3 0,3 -1,0 -1,0 0))))"
    );

    Ok(())
}

#[test]
#[ignore]
fn surface_layers() -> Result<()> {
    assert_eq!(
        &read_layer_geometry("surface/polyhedralsurface.fgb", true)?,
        "POLYHEDRALSURFACE(((0 0 0,0 0 1,0 1 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,1 0 0,0 0 0)),((0 0 0,1 0 0,1 0 1,0 0 1,0 0 0)),((1 1 0,1 1 1,1 0 1,1 0 0,1 1 0)),((0 1 0,0 1 1,1 1 1,1 1 0,0 1 0)),((0 0 1,1 0 1,1 1 1,0 1 1,0 0 1)))"
    );
    assert_eq!(
        &read_layer_geometry("surface/tin.fgb", true)?,
        "TIN(((0 0 0,0 0 1,0 1 0,0 0 0)),((0 0 0,0 1 0,1 1 0,0 0 0)))"
    );
    assert_eq!(
        &read_layer_geometry("surface/triangle.fgb", true)?,
        "TRIANGLE((0 0,0 9,9 0,0 0))"
    );

    Ok(())
}

#[test]
#[ignore]
fn multilinestring_layer() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/ne_10m_geographic_lines.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::MultiLineString);
    assert_eq!(fgb.header().features_count(), 6);
    let _geometry_type = fgb.header().geometry_type();

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::Unknown);
    let mut num_vertices = 0;
    for _i in (0..geometry.xy().unwrap().len()).step_by(2) {
        num_vertices += 1;
    }
    assert_eq!(num_vertices, 361);
    let wkt = feature.to_wkt()?;
    assert_eq!(
        &wkt[0..80],
        "MULTILINESTRING((-20037505.025679983 2692596.21474788,-19924286.672913034 269259"
    );

    Ok(())
}

struct MaxFinder(f64);

impl GeomProcessor for MaxFinder {
    fn dimensions(&self) -> CoordDimensions {
        CoordDimensions::xyz()
    }
    fn coordinate(
        &mut self,
        _x: f64,
        _y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) -> Result<()> {
        if let Some(z) = z {
            if z > self.0 {
                self.0 = z
            }
        }
        Ok(())
    }
}

#[test]
#[ignore]
fn multi_dim() -> Result<()> {
    let mut filein = BufReader::new(File::open(
        "../../test/data/geoz_lod1_gebaeude_max_3d_extract.fgb",
    )?);
    let mut fgb = FgbReader::open(&mut filein)?;
    let geometry_type = fgb.header().geometry_type();
    assert_eq!(geometry_type, GeometryType::MultiPolygon);
    assert_eq!(fgb.header().hasZ(), true);
    assert_eq!(fgb.header().hasM(), false);
    assert_eq!(fgb.header().hasT(), false);
    assert_eq!(fgb.header().hasTM(), false);
    assert_eq!(fgb.header().features_count(), 87);

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), geometry_type);
    // MULTIPOLYGON Z (((2683312.339 1247968.33 401.7,2683311.496 1247964.044 401.7,2683307.761 1247964.745 401.7,2683309.16 1247973.337 401.7,2683313.003 1247972.616 401.7,2683312.339 1247968.33 401.7),(2683312.339 1247968.33
    // 401.7,2683313.003 1247972.616 401.7,2683313.003 1247972.616 410.5,2683312.339 1247968.33 410.5,2683312.339 1247968.33 401.7),(2683307.761 1247964.745 401.7,2683311.496 1247964.044 401.7,2683311.496 1247964.044 410.5,268330
    // 7.761 1247964.745 410.5,2683307.761 1247964.745 401.7),(2683311.496 1247964.044 401.7,2683312.339 1247968.33 401.7,2683312.339 1247968.33 410.5,2683311.496 1247964.044 410.5,2683311.496 1247964.044 401.7)),((2683309.16 124
    // 7973.337 401.7,2683307.761 1247964.745 401.7,2683307.761 1247964.745 410.5,2683309.16 1247973.337 410.5,2683309.16 1247973.337 401.7)),((2683312.339 1247968.33 410.5,2683311.496 1247964.044 410.5,2683307.761 1247964.745 41
    // 0.5,2683309.16 1247973.337 410.5,2683313.003 1247972.616 410.5,2683312.339 1247968.33 410.5),(2683313.003 1247972.616 401.7,2683309.16 1247973.337 401.7,2683309.16 1247973.337 410.5,2683313.003 1247972.616 410.5,2683313.00
    // 3 1247972.616 401.7)))

    let mut max_finder = MaxFinder(0.0);
    geometry.process(&mut max_finder, geometry_type)?;
    assert_eq!(max_finder.0, 410.5);

    let _props = feature.properties()?;

    Ok(())
}

struct PropChecker<'a> {
    expected: Vec<ColumnValue<'a>>,
}

impl PropertyProcessor for PropChecker<'_> {
    fn property(&mut self, i: usize, _name: &str, v: &ColumnValue) -> Result<bool> {
        assert_eq!(v, &self.expected[i]);
        Ok(false)
    }
}

#[test]
fn property_types() -> Result<()> {
    let mut filein = BufReader::new(File::open("../../test/data/alldatatypes.fgb")?);
    let mut fgb = FgbReader::open(&mut filein)?;

    let _count = fgb.select_all()?;
    let feature = fgb.next()?.unwrap();
    let mut prop_checker = PropChecker {
        expected: vec![
            ColumnValue::Byte(-1),
            ColumnValue::UByte(255),
            ColumnValue::Bool(true),
            ColumnValue::Short(-1),
            ColumnValue::UShort(65535),
            ColumnValue::Int(-1),
            ColumnValue::UInt(4294967295),
            ColumnValue::Long(-1),
            ColumnValue::ULong(18446744073709551615),
            ColumnValue::Float(0.0),
            ColumnValue::Double(0.0),
            ColumnValue::String("X"),
            ColumnValue::Json("X"),
            ColumnValue::DateTime("2020-02-29T12:34:56Z"),
            ColumnValue::Binary(&[88]),
        ],
    };
    assert!(feature.process_properties(&mut prop_checker).is_ok());

    Ok(())
}
