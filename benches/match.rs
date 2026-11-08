use criterion::{Criterion, criterion_group, criterion_main};
use real_regex::Regex as RealRegex;
use regex::Regex;
use regex_filtered::Builder;
use regex_lite::Regex as RegexLite;

fn all_matches<'a>(matches: impl Iterator<Item = &'a str>) -> Vec<String> {
    matches.map(str::to_string).collect()
}

fn compare_match(c: &mut Criterion) {
    let input = "the quick brown fox jumps over the lazy dog".to_string();
    let pattern = Regex::new(r"\w+").unwrap();
    let real_pattern = RealRegex::new(r"\w+").unwrap();
    let lite_pattern = RegexLite::new(r"\w+").unwrap();
    let filtered = Builder::new().push(r"\w+").unwrap().build().unwrap();

    let mut group = c.benchmark_group("match");
    group.bench_function("regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("fast_regex", |b| {
        b.iter(|| fast_regex::r#match(std::hint::black_box(input.clone())))
    });
    group.bench_function("real_regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(real_pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("regex_lite", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(lite_pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("regex_filtered", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            filtered
                .matching(haystack)
                .flat_map(|(_, re)| re.find_iter(haystack).map(|m| m.as_str()))
                .map(str::to_string)
                .collect::<Vec<String>>()
        })
    });
    group.finish();
}

criterion_group!(benches, compare_match);
criterion_main!(benches);
