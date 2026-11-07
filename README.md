# Regex in Rust
regexes to search in strings

## Benchmark

One Criterion run of `benches/match.rs`. Each implementation scans `the quick brown fox jumps over the lazy dog` for `\w+` and collects the matches into a `Vec<String>`.

| Implementation | Average time | 95% CI |
| --- | ---: | --- |
| `regex` | 628 ns | 626–630 ns |
| `lazy_regex` | 661 ns | 659–663 ns |
| `regex_filtered` | 936 ns | 935–937 ns |
| `fast_regex` | 29.6 ns | 29.59–29.64 ns |

`fast_regex` currently returns the input string unchanged, so its time is the cost of that return, not a search.

![Violin plot of the match benchmark](benchmarks/violin.svg)
