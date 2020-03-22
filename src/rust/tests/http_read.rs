use flatgeobuf::*;
use tokio::runtime::Runtime;

async fn http_read_async() {
    let client = HttpClient::new("https://pkg.sourcepole.ch/countries.fgb");
    let hreader = HttpHeaderReader::read(&client).await.unwrap();
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(header.features_count(), 179);

    let mut freader = HttpFeatureReader::select_all(&header, hreader.header_len())
        .await
        .unwrap();
    let feature = freader.next(&client).await.unwrap();
    let props = feature.properties_map(&header);
    assert_eq!(props["name"], "Antarctica".to_string());
}

#[test]
fn http_read() {
    Runtime::new().unwrap().block_on(http_read_async());
}

#[allow(dead_code)]
async fn http_svg_async() {
    let client = HttpClient::new("https://pkg.sourcepole.ch/countries.fgb");
    let hreader = HttpHeaderReader::read(&client).await.unwrap();
    let header = hreader.header();

    let mut freader = HttpFeatureReader::select_all(&header, hreader.header_len())
        .await
        .unwrap();
    let mut svg_data: Vec<u8> = Vec::new();
    freader
        .to_svg(&client, &header, 800, 400, &mut svg_data)
        .await
        .unwrap();
}

// #[test]
// fn http_svg() {
//     Runtime::new().unwrap().block_on(http_svg_async());
// }

fn result_err_str<T>(res: Result<T, std::io::Error>) -> String {
    match res {
        Ok(_) => String::new(),
        Err(e) => format!("{}", e),
    }
}

async fn http_err_async() {
    let client = HttpClient::new("http://pkg.sourcepole.ch/wrong.fgb");
    let hreader = HttpHeaderReader::read(&client).await;
    assert_eq!(
        result_err_str(hreader),
        "Response with status 404 Not Found".to_string()
    );
    let client = HttpClient::new("http://wrong.sourcepole.ch/countries.fgb");
    let hreader = HttpHeaderReader::read(&client).await;
    assert_eq!(result_err_str(hreader), "error sending request for url (http://wrong.sourcepole.ch/countries.fgb): error trying to connect: dns error: failed to lookup address information: Name or service not known".to_string());
}

#[test]
fn http_err() {
    Runtime::new().unwrap().block_on(http_err_async());
}
