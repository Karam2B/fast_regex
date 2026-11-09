# Regex in Rust
regexes to search in strings

## Benchmark

One Criterion run of `benches/match.rs`. Each implementation scans `the quick brown fox jumps over the lazy dog` for `\w+` and collects the matches into a `Vec<String>`.

| Implementation | Average time | 95% CI |
| --- | ---: | --- |
| `ideal_matching` | 171.85 ns | — |
| my crate WILL have the same performance as ideal matching |
| `fast_regex` | 171ns | — |
| `regex` | 628 ns | 627–628 ns |
| `real_regex` | 651 ns | 651–652 ns |
| `regex_filtered` | 878 ns | 876–879 ns |
| `regex_lite` | 2.18 µs | 2.17–2.18 µs |
 
