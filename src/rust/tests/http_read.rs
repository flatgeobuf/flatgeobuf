use flatgeobuf::*;

#[test]
fn header_http() -> std::result::Result<(), std::io::Error> {
    let client = HttpClient::new("http://pkg.sourcepole.ch/countries.fgb");
    let hreader = HttpHeaderReader::read(&client)?;
    let header = hreader.header();
    assert_eq!(header.geometry_type(), GeometryType::MultiPolygon);
    assert_eq!(header.features_count(), 179);
    Ok(())
}

fn result_err_str<T>(res: Result<T, std::io::Error>) -> String {
    match res {
        Ok(_) => String::new(),
        Err(e) => format!("{}", e),
    }
}

#[test]
fn header_http_err() {
    let client = HttpClient::new("http://pkg.sourcepole.ch/wrong.fgb");
    let hreader = HttpHeaderReader::read(&client);
    assert_eq!(
        result_err_str(hreader),
        "Response with status 404 Not Found".to_string()
    );
    let client = HttpClient::new("http://wrong.sourcepole.ch/countries.fgb");
    let hreader = HttpHeaderReader::read(&client);
    assert_eq!(result_err_str(hreader), "error sending request for url (http://wrong.sourcepole.ch/countries.fgb): error trying to connect: dns error: failed to lookup address information: Name or service not known".to_string());
}
