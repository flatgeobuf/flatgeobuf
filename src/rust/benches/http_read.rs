use criterion::{criterion_group, criterion_main, Criterion};

use flatgeobuf::HttpFgbReader;
use geozero::error::Result;

// 205KB
const SMALL_URL: &str = "http://localhost:8001/countries.fgb";

// 13MB
const MEDIUM_URL: &str = "http://localhost:8001/UScounties.fgb";

async fn select_all(url: &str, expected_feature_count: usize) -> Result<()> {
    let reader = HttpFgbReader::open(url).await.unwrap();
    let mut stream = reader.select_all().await.unwrap();

    let mut count = 0;
    while let Some(feature) = stream.next().await.transpose() {
        let _feature = feature.unwrap();
        count += 1
    }
    assert_eq!(count, expected_feature_count);
    Ok(())
}

async fn select_bbox(url: &str, expected_feature_count: usize) {
    let reader = HttpFgbReader::open(url).await.unwrap();
    let mut stream = reader.select_bbox(-86.0, 10.0, -85.0, 40.0).await.unwrap();

    let mut count = 0;
    while let Some(feature) = stream.next().await.transpose() {
        let _feature = feature.unwrap();
        count += 1
    }
    assert_eq!(count, expected_feature_count);
}

fn criterion_benchmark(c: &mut Criterion) {
    use std::time::Duration;
    use yocalhost::ThrottledServer;

    let port = 8001;

    // Apply limits to simulate an "average" connection for benchmarks
    let latency = Duration::from_millis(100);
    let bytes_per_second = 50_000_000 / 8;

    let web_root = "../../test/data";
    let server = ThrottledServer::new(port, latency, bytes_per_second, web_root);

    let runtime = tokio::runtime::Runtime::new().unwrap();
    runtime.spawn(async move {
        server.serve().await;
    });

    for (name, url, total_count, bbox_count) in [
        ("small", SMALL_URL, 179, 4),
        ("medium", MEDIUM_URL, 3221, 140),
    ] {
        c.bench_function(&format!("{name} select_all"), |b| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            b.to_async(runtime)
                .iter(|| select_all(url, total_count))
        });

        c.bench_function(&format!("{name} select_bbox"), |b| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            b.to_async(runtime)
                .iter(|| select_bbox(url, bbox_count))
        });
    }
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10); targets=criterion_benchmark);
criterion_main!(benches);
