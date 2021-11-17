use criterion::{criterion_group, criterion_main, Bencher, BenchmarkId, Criterion};
use futures::{pin_mut, Stream, StreamExt};
use std::pin::Pin;
use tokio::runtime::Runtime;

fn cache(c: &mut Criterion) {
    let mut g = c.benchmark_group("cache");
    g.sample_size(10);
    let n: usize = 1000000;
    g.bench_with_input(BenchmarkId::new("Cache1", n.to_string()), &n, |b, &n| {
        cache1(Runtime::new().unwrap(), b, n)
    });
    g.bench_with_input(BenchmarkId::new("Cache2", n.to_string()), &n, |b, &n| {
        cache2(Runtime::new().unwrap(), b, n)
    });
    g.bench_with_input(
        BenchmarkId::new("CacheChunk", n.to_string()),
        &n,
        |b, &n| cache_chunk(Runtime::new().unwrap(), b, n, 20000)
    );
    g.finish();
}

fn cache1(rt: Runtime, b: &mut Bencher, n: usize) {
    b.iter(|| {
        rt.handle().block_on(async {
            let stream = stream(n);
            let c = bench_cache::cache::CacheStream::new(stream);
            let c2 = c.reload();
            let result = tokio::join!(
                rt.spawn(async {
                    pin_mut!(c);
                    while c.next().await.is_some() {}
                }),
                rt.spawn(async {
                    pin_mut!(c2);
                    while c2.next().await.is_some() {}
                }),
            );
            result.0.unwrap();
            result.1.unwrap();
        });
    });
}

fn cache2(rt: Runtime, b: &mut Bencher, n: usize) {
    b.iter(|| {
        rt.handle().block_on(async {
            let stream = stream(n);
            let (load, c) = bench_cache::cache2::CacheStream::new(rt.handle().clone(), stream);
            let c2 = c.new_reload_only();
            let result = tokio::join!(
                rt.spawn(load),
                rt.spawn(async {
                    pin_mut!(c);
                    while c.next().await.is_some() {}
                }),
                rt.spawn(async {
                    pin_mut!(c2);
                    while c2.next().await.is_some() {}
                }),
            );
            result.0.unwrap();
            result.1.unwrap();
            result.2.unwrap();
        });
    });
}

fn cache_chunk(rt: Runtime, b: &mut Bencher, n: usize, chunk: usize) {
    b.iter(|| {
        rt.handle().block_on(async {
            let stream = stream(n);
            let (l, c) = bench_cache::cache_chunks::new_cache_chunks(stream, 100, chunk);
            let c2 = c.renew();
            let result = tokio::join!(
                rt.spawn(l),
                rt.spawn(async {
                    pin_mut!(c);
                    while c.next().await.is_some() {}
                }),
                rt.spawn(async {
                    pin_mut!(c2);
                    while c2.next().await.is_some() {}
                }),
            );
            result.0.unwrap();
            result.1.unwrap();
            result.2.unwrap();
        });
    });
}

fn stream(n: usize) -> Pin<Box<dyn Stream<Item = (String, Vec<u16>)> + Send + Sync + 'static>> {
    let s = Box::pin(futures::stream::unfold(0..n, |mut it| async {
        it.next().map(|i| (i.to_string(), it))
    }));
    let f = Box::pin(s.map(|x| {
        let score = if x.find('0').is_some() {
            vec![1]
        } else {
            vec![]
        };
        (x, score)
    }));
    f
}

criterion_group!(benches, cache);
criterion_main!(benches);
