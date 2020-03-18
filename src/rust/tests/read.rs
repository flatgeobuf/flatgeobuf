use flatgeobuf::*;
use std::fs::File;
use std::io::{BufReader, Read, Seek, SeekFrom};

#[test]
fn read_file_low_level() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/countries.fgb")?;
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
    let index_size =
        PackedRTree::index_size(header.features_count() as usize, header.index_node_size());
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
        for _j in (0..part.xy().unwrap().len()).step_by(2) {
            num_vertices += 1;
        }
    }
    assert_eq!(num_vertices, 658);

    assert!(feature.properties().is_some());
    assert!(feature.columns().is_none());
    Ok(())
}

struct VertexCounter(u64);

impl GeomReader for VertexCounter {
    fn pointxy(&mut self, _x: f64, _y: f64, _idx: usize) {
        self.0 += 1;
    }
}

#[test]
fn file_reader() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/countries.fgb")?;
    let mut reader = BufReader::new(f);
    let hreader = HeaderReader::read(&mut reader)?;
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(header.features_count(), 179);

    let mut freader = FeatureReader::select_all(&mut reader, &header)?;
    assert_eq!(freader.filter_count(), None);

    while let Ok(feature) = freader.next(&mut reader) {
        let found = feature.iter_properties(&header, |i, _n, v| {
            i == 0 && v == ColumnValue::String("DNK")
        });
        if found {
            break;
        }
    }
    let feature = freader.cur_feature();
    // OGRFeature(countries):46
    //   id (String) = DNK
    //   name (String) = Denmark
    //   MULTIPOLYGON (((12.690006 55.609991,12.089991 54.800015,11.043543 55.364864,10.903914 55.779955,12.370904 56.111407,12.690006 55.609991)),((10.912182 56.458621,1
    // 0.667804 56.081383,10.369993 56.190007,9.649985 55.469999,9.921906 54.983104,9.282049 54.830865,8.526229 54.962744,8.120311 55.517723,8.089977 56.540012,8.256582 5
    // 6.809969,8.543438 57.110003,9.424469 57.172066,9.775559 57.447941,10.580006 57.730017,10.546106 57.215733,10.25 56.890016,10.369993 56.609982,10.912182 56.458621))
    // )
    let geometry = feature.geometry().unwrap();

    let mut vertex_counter = VertexCounter(0);
    geometry.parse(&mut vertex_counter, header.geometry_type());
    assert_eq!(vertex_counter.0, 24);

    let props = feature.properties_map(&header);
    assert_eq!(props["id"], "DNK".to_string());
    assert_eq!(props["name"], "Denmark".to_string());

    Ok(())
}

#[test]
fn bbox_file_reader() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/countries.fgb")?;
    let mut reader = BufReader::new(f);
    let hreader = HeaderReader::read(&mut reader)?;
    let header = hreader.header();

    let mut freader = FeatureReader::select_bbox(&mut reader, &header, 8.8, 47.2, 9.5, 55.3)?;
    assert_eq!(freader.filter_count(), Some(6));

    let feature = freader.next(&mut reader).unwrap();
    let props = feature.properties_map(&header);
    assert_eq!(props["name"], "Denmark".to_string());

    Ok(())
}

#[test]
fn magic_byte() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/states.geojson")?;
    let mut reader = BufReader::new(f);
    assert_eq!(
        HeaderReader::read(&mut reader).err().unwrap().to_string(),
        "Magic byte doesn\'t match"
    );

    Ok(())
}

#[test]
#[ignore]
fn point_layer() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/ne_10m_admin_0_country_points.fgb")?;
    let mut reader = BufReader::new(f);
    let hreader = HeaderReader::read(&mut reader)?;
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::Point);
    assert_eq!(header.features_count(), 250);

    let mut freader = FeatureReader::select_all(&mut reader, &header)?;
    let feature = freader.next(&mut reader)?;
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::Unknown);
    let xy = geometry.xy().unwrap();
    assert_eq!(
        (xy.get(0), xy.get(1)),
        (2223639.4731508396, -15878634.348995442)
    );
    let _props = feature.properties_map(&header);

    Ok(())
}

struct WktLineEmitter {
    wkt: String,
}

impl GeomReader for WktLineEmitter {
    fn line_begin(&mut self, _n: usize, _idx: usize) {
        self.wkt.push_str("LINESTRING (");
    }
    fn pointxy(&mut self, x: f64, y: f64, idx: usize) {
        if idx > 0 {
            self.wkt.push_str(", ");
        }
        self.wkt.push_str(&format!("{} {}", x, y));
    }
    fn line_end(&mut self, _idx: usize) {
        self.wkt.push_str(")");
    }
}

#[test]
#[ignore]
fn line_layer() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/lines.fgb")?;
    let mut reader = BufReader::new(f);
    let hreader = HeaderReader::read(&mut reader)?;
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::LineString);
    assert_eq!(header.features_count(), 8375);

    let mut freader = FeatureReader::select_all(&mut reader, &header)?;
    let feature = freader.next(&mut reader)?;
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

    let mut visitor = WktLineEmitter { wkt: String::new() };
    geometry.parse(&mut visitor, header.geometry_type());
    assert_eq!(visitor.wkt, "LINESTRING (1875038.4476102313 -3269648.6879248763, 1874359.6415041967 -3270196.8129848638, 1874141.0428635243 -3270953.7840121365, 1874440.1778162003 -3271619.4315206874, 1876396.0598222911 -3274138.747656357, 1876442.0805243007 -3275052.60551469, 1874739.312657555 -3275457.333765534)");

    let _props = feature.properties_map(&header);

    Ok(())
}

struct MultiLineGenerator(Vec<Vec<(f64, f64)>>);

impl GeomReader for MultiLineGenerator {
    fn multiline_begin(&mut self, n: usize, _idx: usize) {
        self.0.reserve(n);
    }
    fn ring_begin(&mut self, n: usize, _idx: usize) {
        self.0.push(Vec::with_capacity(n));
    }
    fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
        let len = self.0.len();
        self.0[len - 1].push((x, y));
    }
}

#[test]
#[ignore]
fn multi_line_layer() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/ne_10m_geographic_lines.fgb")?;
    let mut reader = BufReader::new(f);
    let hreader = HeaderReader::read(&mut reader)?;
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::MultiLineString);
    assert_eq!(header.features_count(), 6);

    let mut freader = FeatureReader::select_all(&mut reader, &header)?;
    let feature = freader.next(&mut reader)?;
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), GeometryType::Unknown);
    let mut num_vertices = 0;
    for _i in (0..geometry.xy().unwrap().len()).step_by(2) {
        num_vertices += 1;
    }
    assert_eq!(num_vertices, 361);

    let mut visitor = MultiLineGenerator(Vec::new());
    geometry.parse(&mut visitor, header.geometry_type());
    assert_eq!(visitor.0.len(), 1);
    assert_eq!(visitor.0[0].len(), 361);
    assert_eq!(visitor.0[0][0], (-20037505.025679983, 2692596.21474788));

    let _props = feature.properties_map(&header);

    Ok(())
}

struct MaxFinder(f64);

impl GeomReader for MaxFinder {
    fn dimensions(&self) -> Dimensions {
        Dimensions {
            z: true,
            m: false,
            t: false,
            tm: false,
        }
    }
    fn point(
        &mut self,
        _x: f64,
        _y: f64,
        z: Option<f64>,
        _m: Option<f64>,
        _t: Option<f64>,
        _tm: Option<u64>,
        _idx: usize,
    ) {
        if let Some(z) = z {
            if z > self.0 {
                self.0 = z
            }
        }
    }
}

#[test]
#[ignore]
fn multi_dim() -> std::result::Result<(), std::io::Error> {
    let f = File::open("../../test/data/geoz_lod1_gebaeude_max_3d_extract.fgb")?;
    let mut reader = BufReader::new(f);
    let hreader = HeaderReader::read(&mut reader)?;
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(header.hasZ(), true);
    assert_eq!(header.hasM(), false);
    assert_eq!(header.hasT(), false);
    assert_eq!(header.hasTM(), false);
    assert_eq!(header.features_count(), 87);

    let mut freader = FeatureReader::select_all(&mut reader, &header)?;
    let feature = freader.next(&mut reader)?;
    assert!(feature.geometry().is_some());
    let geometry = feature.geometry().unwrap();
    assert_eq!(geometry.type_(), header.geometry_type());
    // MULTIPOLYGON Z (((2683312.339 1247968.33 401.7,2683311.496 1247964.044 401.7,2683307.761 1247964.745 401.7,2683309.16 1247973.337 401.7,2683313.003 1247972.616 401.7,2683312.339 1247968.33 401.7),(2683312.339 1247968.33
    // 401.7,2683313.003 1247972.616 401.7,2683313.003 1247972.616 410.5,2683312.339 1247968.33 410.5,2683312.339 1247968.33 401.7),(2683307.761 1247964.745 401.7,2683311.496 1247964.044 401.7,2683311.496 1247964.044 410.5,268330
    // 7.761 1247964.745 410.5,2683307.761 1247964.745 401.7),(2683311.496 1247964.044 401.7,2683312.339 1247968.33 401.7,2683312.339 1247968.33 410.5,2683311.496 1247964.044 410.5,2683311.496 1247964.044 401.7)),((2683309.16 124
    // 7973.337 401.7,2683307.761 1247964.745 401.7,2683307.761 1247964.745 410.5,2683309.16 1247973.337 410.5,2683309.16 1247973.337 401.7)),((2683312.339 1247968.33 410.5,2683311.496 1247964.044 410.5,2683307.761 1247964.745 41
    // 0.5,2683309.16 1247973.337 410.5,2683313.003 1247972.616 410.5,2683312.339 1247968.33 410.5),(2683313.003 1247972.616 401.7,2683309.16 1247973.337 401.7,2683309.16 1247973.337 410.5,2683313.003 1247972.616 410.5,2683313.00
    // 3 1247972.616 401.7)))

    let mut max_finder = MaxFinder(0.0);
    geometry.parse(&mut max_finder, header.geometry_type());
    assert_eq!(max_finder.0, 410.5);

    let _props = feature.properties_map(&header);

    Ok(())
}
