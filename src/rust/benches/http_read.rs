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
    let latency = {
        // e.g.
        // FGB_BENCH_LATENCY_MS=50 cargo bench --bench http_read
        let millis = std::env::var("FGB_BENCH_LATENCY_MS")
            .map(|str| {
                str.parse::<u64>()
                    .expect("FGB_BENCH_LATENCY_MS must be an integer")
            })
            .unwrap_or(100);
        Duration::from_millis(millis)
    };

    // e.g.
    // 1 gigabit: FGB_BENCH_BYTES_PER_SEC=125000000 cargo bench --bench http_read
    //   10 mbit: FGB_BENCH_BYTES_PER_SEC=1250000 cargo bench --bench http_read
    let bytes_per_second = {
        std::env::var("FGB_BENCH_BYTES_PER_SEC")
            .map(|str| {
                str.parse::<u64>()
                    .expect("FGB_BENCH_BYTES_PER_SEC must be an integer")
            })
            .unwrap_or(50_000_000 / 8)
    };

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
            b.to_async(runtime).iter(|| select_all(url, total_count))
        });

        c.bench_function(&format!("{name} select_bbox"), |b| {
            let runtime = tokio::runtime::Runtime::new().unwrap();
            b.to_async(runtime).iter(|| select_bbox(url, bbox_count))
        });
    }
}

criterion_group!(name=benches; config=Criterion::default().sample_size(10); targets=criterion_benchmark);
criterion_main!(benches);
