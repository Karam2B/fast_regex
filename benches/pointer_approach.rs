use criterion::{Criterion, criterion_group, criterion_main};
use real_regex::Regex as RealRegex;
use regex::Regex;
use regex_filtered::Builder;
use regex_lite::Regex as RegexLite;

fn all_matches<'a>(matches: impl Iterator<Item = &'a str>) -> Vec<String> {
    matches.map(str::to_string).collect()
}

fn compare_el(c: &mut Criterion) {
    let input = "hello woeld".to_string();
    let pattern = Regex::new("el").unwrap();
    let real_pattern = RealRegex::new("el").unwrap();
    let lite_pattern = RegexLite::new("el").unwrap();
    let filtered = Builder::new().push("el").unwrap().build().unwrap();

    let mut group = c.benchmark_group("many_matches_el");
    group.bench_function("regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("many_matches_el", |b| {
        b.iter(|| {
            all_matches(fast_regex::many_matches_el(std::hint::black_box(&input)).into_iter())
        })
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

fn compare_repeat(c: &mut Criterion) {
    let input = "1___2  1__2   1____2".to_string();
    let pattern = Regex::new(r"1(?:__)*2").unwrap();
    let real_pattern = RealRegex::new(r"1(?:__)*2").unwrap();
    let lite_pattern = RegexLite::new(r"1(?:__)*2").unwrap();
    let filtered = Builder::new().push(r"1(?:__)*2").unwrap().build().unwrap();

    let mut group = c.benchmark_group("many_on_repeat");
    group.bench_function("regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("many_on_repeat", |b| {
        b.iter(|| all_matches(fast_regex::many_on_repeat(std::hint::black_box(&input)).into_iter()))
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

fn compare_repeat_2(c: &mut Criterion) {
    let input = "1___2  1__2   1____2".to_string();
    let pattern = Regex::new(r"(?:__)*2").unwrap();
    let real_pattern = RealRegex::new(r"(?:__)*2").unwrap();
    let lite_pattern = RegexLite::new(r"(?:__)*2").unwrap();
    let filtered = Builder::new().push(r"(?:__)*2").unwrap().build().unwrap();

    let mut group = c.benchmark_group("many_on_repeat_2");
    group.bench_function("regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("many_on_repeat_2", |b| {
        b.iter(|| {
            all_matches(fast_regex::many_on_repeat_2(std::hint::black_box(&input)).into_iter())
        })
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

fn compare_repeat_3(c: &mut Criterion) {
    let input = "2  1__2   1____2".to_string();
    let pattern = Regex::new(r"(?:__)?2").unwrap();
    let real_pattern = RealRegex::new(r"(?:__)?2").unwrap();
    let lite_pattern = RegexLite::new(r"(?:__)?2").unwrap();
    let filtered = Builder::new().push(r"(?:__)?2").unwrap().build().unwrap();

    let mut group = c.benchmark_group("many_on_repeat_3");
    group.bench_function("regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("many_on_repeat_3", |b| {
        b.iter(|| {
            all_matches(fast_regex::many_on_repeat_3(std::hint::black_box(&input)).into_iter())
        })
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

fn compare_repeat_4(c: &mut Criterion) {
    let input = "1__2  1____2   1______2  1________2".to_string();
    let pattern = Regex::new(r"(?:__){2,3}2").unwrap();
    let real_pattern = RealRegex::new(r"(?:__){2,3}2").unwrap();
    let lite_pattern = RegexLite::new(r"(?:__){2,3}2").unwrap();
    let filtered = Builder::new()
        .push(r"(?:__){2,3}2")
        .unwrap()
        .build()
        .unwrap();

    let mut group = c.benchmark_group("many_on_repeat_4");
    group.bench_function("regex", |b| {
        b.iter(|| {
            let haystack = std::hint::black_box(&input);
            all_matches(pattern.find_iter(haystack).map(|m| m.as_str()))
        })
    });
    group.bench_function("many_on_repeat_4", |b| {
        b.iter(|| {
            all_matches(fast_regex::many_on_repeat_4(std::hint::black_box(&input)).into_iter())
        })
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

criterion_group!(
    benches,
    compare_el,
    compare_repeat,
    compare_repeat_2,
    compare_repeat_3,
    compare_repeat_4
);
criterion_main!(benches);
