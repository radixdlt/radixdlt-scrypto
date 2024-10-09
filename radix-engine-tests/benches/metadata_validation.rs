use criterion::{criterion_group, criterion_main, Criterion};
use radix_common::prelude::*;
use radix_engine::object_modules::metadata::MetadataValidationError;
use scrypto::prelude::{CheckedUrl, MetadataCreateWithDataInput, MetadataInit, UncheckedUrl};
use std::hint::black_box;

#[allow(unused_must_use)]
fn bench_validate_urls(c: &mut Criterion) {
    let mut urls: Vec<UncheckedUrl> = vec![UncheckedUrl::of("https://www.google.com")];
    for i in 0..25_000 {
        urls.push(UncheckedUrl::of(format!("https://www.google.com/{i}?q=x")));
    }
    urls.push(UncheckedUrl::of("asdf"));

    let mut data = MetadataInit::default();
    data.set_metadata("urls", urls.clone());
    let args = scrypto_encode(&MetadataCreateWithDataInput { data }).unwrap();
    std::fs::write("/tmp/args.bin", args).unwrap();

    c.bench_function("metadata_validation::validate_urls", |b| {
        b.iter(|| {
            for url in &urls {
                black_box(
                    CheckedUrl::of(url.as_str())
                        .ok_or(MetadataValidationError::InvalidURL(url.as_str().to_owned())),
                );
            }
        })
    });
}

criterion_group!(metadata_validation, bench_validate_urls);
criterion_main!(metadata_validation);
