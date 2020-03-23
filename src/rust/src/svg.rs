use crate::feature_generated::flat_geobuf::{Feature, Geometry};
use crate::file_reader::FeatureReader;
use crate::geometry_reader::GeomReader;
use crate::header_generated::flat_geobuf::{GeometryType, Header};
use crate::http_reader::{BufferedHttpClient, HttpFeatureReader};
use std::io::{Read, Seek, Write};

struct SvgEmitter<'a, W: Write> {
    out: &'a mut W,
    invert_y: bool,
}

impl<'a, W: Write> SvgEmitter<'a, W> {
    fn new(out: &'a mut W, invert_y: bool) -> SvgEmitter<'a, W> {
        SvgEmitter { out, invert_y }
    }
}

impl<W: Write> GeomReader for SvgEmitter<'_, W> {
    fn pointxy(&mut self, x: f64, y: f64, _idx: usize) {
        let y = if self.invert_y { -y } else { y };
        self.out.write(&format!("{} {} ", x, y).as_bytes()).unwrap();
    }
    fn point_begin(&mut self, _idx: usize) {
        self.out.write(br#"<path d="M "#).unwrap();
    }
    fn point_end(&mut self) {
        self.out.write(br#"Z"/>"#).unwrap();
    }
    fn line_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn line_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn multiline_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn multiline_end(&mut self) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn ring_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(b"M ").unwrap();
    }
    fn ring_end(&mut self, _idx: usize) {
        self.out.write(b"Z ").unwrap();
    }
    fn poly_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn poly_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
    fn subpoly_begin(&mut self, _size: usize, _idx: usize) {
        self.out.write(br#"<path d=""#).unwrap();
    }
    fn subpoly_end(&mut self, _idx: usize) {
        self.out.write(br#""/>"#).unwrap();
    }
}

impl Geometry<'_> {
    pub fn to_svg<'a, W: Write>(
        &self,
        mut out: &'a mut W,
        geometry_type: GeometryType,
        invert_y: bool,
    ) {
        let mut svg = SvgEmitter::new(&mut out, invert_y);
        self.parse(&mut svg, geometry_type);
    }
}

impl Feature<'_> {
    /// Convert feature to SVG
    pub fn to_svg<'a, W: Write>(
        &self,
        mut out: &'a mut W,
        geometry_type: GeometryType,
        invert_y: bool,
    ) {
        let mut svg = SvgEmitter::new(&mut out, invert_y);
        let geometry = self.geometry().unwrap();
        geometry.parse(&mut svg, geometry_type);
    }
}

impl FeatureReader {
    /// Convert selected FlatGeoBuf features to SVG
    ///
    /// Usage:
    ///```rust
    /// # use flatgeobuf::*;
    /// # use std::fs::File;
    /// # use std::io::{BufReader, BufWriter};
    /// # fn fgb_to_svg() -> std::result::Result<(), std::io::Error> {
    /// # let mut filein = BufReader::new(File::open("countries.fgb")?);
    /// # let hreader = HeaderReader::read(&mut filein)?;
    /// # let header = hreader.header();
    /// let mut freader = FeatureReader::select_all(&mut filein, &header)?;
    /// let mut fileout = BufWriter::new(File::create("countries.svg")?);
    /// freader.to_svg(&mut filein, &header, 800, 400, &mut fileout)
    /// # }
    ///```
    pub fn to_svg<'a, R: Read + Seek, W: Write>(
        &mut self,
        mut reader: R,
        header: &Header,
        width: u32,
        height: u32,
        mut out: &'a mut W,
    ) -> std::result::Result<(), std::io::Error> {
        let mut invert_y = false;
        if let Some(crs) = header.crs() {
            if crs.code() == 4326 {
                invert_y = true
            }
        }
        out.write(
            br#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.2" baseProfile="tiny" "#,
        )?;
        out.write(&format!("width=\"{}\" height=\"{}\" ", width, height).as_bytes())
            .unwrap();
        if let Some(envelope) = header.envelope() {
            let (xmin, mut ymin) = (envelope.get(0), envelope.get(1));
            let (xmax, mut ymax) = (envelope.get(2), envelope.get(3));
            if invert_y {
                ymin = -envelope.get(3);
                ymax = -envelope.get(1);
            }
            out.write(
                &format!(
                    "viewBox=\"{} {} {} {}\" ",
                    xmin,
                    ymin,
                    xmax - xmin,
                    ymax - ymin
                )
                .as_bytes(),
            )
            .unwrap();
        }
        out.write(
            br#"stroke-linecap="round" stroke-linejoin="round">
<g id=""#,
        )?;
        if let Some(name) = header.name() {
            out.write(name.as_bytes())?;
        }
        out.write(br#"">"#)?;
        while let Ok(feature) = self.next(&mut reader) {
            out.write(b"\n")?;
            feature.to_svg(&mut out, header.geometry_type(), invert_y);
        }
        out.write(b"\n</g>\n</svg>")?;
        Ok(())
    }
}

impl HttpFeatureReader {
    /// Convert selected FlatGeoBuf features to SVG
    ///
    /// Usage:
    ///```rust
    /// # use flatgeobuf::*;
    /// # use std::fs::File;
    /// # use std::io::BufWriter;
    /// # async fn fgb_to_svg() {
    /// # let mut client = BufferedHttpClient::new("https://pkg.sourcepole.ch/countries.fgb");
    /// # let hreader = HttpHeaderReader::read(&mut client).await.unwrap();
    /// # let header = hreader.header();
    /// let mut freader = HttpFeatureReader::select_all(&header, hreader.header_len()).await.unwrap();
    /// let mut fileout = BufWriter::new(File::create("countries.svg").unwrap());
    /// freader.to_svg(&mut client, &header, 800, 400, &mut fileout);
    /// # }
    ///```
    pub async fn to_svg<'a, W: Write>(
        &mut self,
        client: &mut BufferedHttpClient<'_>,
        header: &Header<'_>,
        width: u32,
        height: u32,
        mut out: &'a mut W,
    ) -> std::result::Result<(), std::io::Error> {
        let mut invert_y = false;
        if let Some(crs) = header.crs() {
            if crs.code() == 4326 {
                invert_y = true
            }
        }
        out.write(
            br#"<?xml version="1.0"?>
<svg xmlns="http://www.w3.org/2000/svg" version="1.2" baseProfile="tiny" "#,
        )?;
        out.write(&format!("width=\"{}\" height=\"{}\" ", width, height).as_bytes())
            .unwrap();
        if let Some(envelope) = header.envelope() {
            let (xmin, mut ymin) = (envelope.get(0), envelope.get(1));
            let (xmax, mut ymax) = (envelope.get(2), envelope.get(3));
            if invert_y {
                ymin = -envelope.get(3);
                ymax = -envelope.get(1);
            }
            out.write(
                &format!(
                    "viewBox=\"{} {} {} {}\" ",
                    xmin,
                    ymin,
                    xmax - xmin,
                    ymax - ymin
                )
                .as_bytes(),
            )
            .unwrap();
        }
        out.write(
            br#"stroke-linecap="round" stroke-linejoin="round">
<g id=""#,
        )?;
        if let Some(name) = header.name() {
            out.write(name.as_bytes())?;
        }
        out.write(br#"">"#)?;
        while let Ok(feature) = self.next(client).await {
            out.write(b"\n")?;
            feature.to_svg(&mut out, header.geometry_type(), invert_y);
        }
        out.write(b"\n</g>\n</svg>")?;
        Ok(())
    }
}
