use criterion::{Criterion, criterion_group, criterion_main};
use lazy_regex::regex;
use regex::Regex;
use regex_filtered::Builder;

fn all_matches(pattern: &Regex, input: &str) -> Vec<String> {
    pattern
        .find_iter(input)
        .map(|m| m.as_str().to_string())
        .collect()
}

fn compare_match(c: &mut Criterion) {
    let input = "the quick brown fox jumps over the lazy dog".to_string();
    let pattern = Regex::new(r"\w+").unwrap();
    let lazy_pattern = regex!(r"\w+");
    let filtered = Builder::new().push(r"\w+").unwrap().build().unwrap();

    let mut group = c.benchmark_group("match");
    group.bench_function("regex", |b| {
        b.iter(|| all_matches(&pattern, std::hint::black_box(&input)))
    });
    group.bench_function("fast_regex", |b| {
        b.iter(|| fast_regex::r#match(std::hint::black_box(input.clone())))
    });
    group.bench_function("lazy_regex", |b| {
        b.iter(|| all_matches(lazy_pattern, std::hint::black_box(&input)))
    });
    group.bench_function("regex_filtered", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            filtered
                .matching(haystack)
                .flat_map(|(_, re)| re.find_iter(haystack).map(|m| m.as_str().to_string()))
                .collect::<Vec<String>>()
        })
    });
    group.finish();
}

criterion_group!(benches, compare_match);
criterion_main!(benches);
