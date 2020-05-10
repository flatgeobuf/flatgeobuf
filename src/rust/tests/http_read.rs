use flatgeobuf::*;
use geozero::error::Result;
use tokio::runtime::Runtime;

async fn http_read_async() -> Result<()> {
    let url =
        "https://raw.githubusercontent.com/bjornharrtell/flatgeobuf/master/test/data/countries.fgb";
    let mut fgb = HttpFgbReader::open(url).await?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(fgb.header().features_count(), 179);
    fgb.select_all().await?;
    let feature = fgb.next().await?.unwrap();
    let props = feature.properties()?;
    assert_eq!(props["name"], "Antarctica".to_string());
    Ok(())
}

#[test]
fn http_read() {
    assert!(Runtime::new().unwrap().block_on(http_read_async()).is_ok());
}

async fn http_bbox_read_async() -> Result<()> {
    let url =
        "https://raw.githubusercontent.com/bjornharrtell/flatgeobuf/master/test/data/countries.fgb";
    let mut fgb = HttpFgbReader::open(url).await?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(fgb.header().features_count(), 179);
    fgb.select_bbox(8.8, 47.2, 9.5, 55.3).await?;
    let feature = fgb.next().await?.unwrap();
    let props = feature.properties()?;
    assert_eq!(props["name"], "Denmark".to_string());
    Ok(())
}

#[test]
fn http_bbox_read() {
    assert!(Runtime::new()
        .unwrap()
        .block_on(http_bbox_read_async())
        .is_ok());
}

async fn http_bbox_big_async() -> Result<()> {
    let url = "https://pkg.sourcepole.ch/osm-buildings-ch.fgb";
    let mut fgb = HttpFgbReader::open(url).await?;
    assert_eq!(fgb.header().geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(fgb.header().features_count(), 2396905);
    fgb.select_bbox(8.522086, 47.363333, 8.553521, 47.376020)
        .await?;
    let feature = fgb.next().await?.unwrap();
    let props = feature.properties()?;
    assert_eq!(props["building"], "residential".to_string());
    Ok(())
}

#[test]
#[ignore]
fn http_bbox_big() {
    assert!(Runtime::new()
        .unwrap()
        .block_on(http_bbox_big_async())
        .is_ok());
}

async fn http_err_async() {
    let url =
        "https://raw.githubusercontent.com/bjornharrtell/flatgeobuf/master/test/data/wrong.fgb";
    let fgb = HttpFgbReader::open(url).await;
    assert_eq!(
        fgb.err().unwrap().to_string(),
        "http status 404".to_string()
    );
    let url = "http://wrong.sourcepole.ch/countries.fgb";
    let fgb = HttpFgbReader::open(url).await;
    assert_eq!(fgb.err()
            .unwrap()
            .to_string(), "http error `error sending request for url (http://wrong.sourcepole.ch/countries.fgb): error trying to connect: dns error: failed to lookup address information: Name or service not known`".to_string());
}

#[test]
fn http_err() {
    Runtime::new().unwrap().block_on(http_err_async());
}
