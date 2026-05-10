# Deferred Optimizations for iratxo-core

## Cold-Start Performance (Single-Shot Benchmark)
The repeated-evaluation benchmark is 93.7% faster than baseline (0.04µs vs 0.63µs), but single-shot cold-start is ~1000x slower (~76µs). Future work:

- **Parallel entity detection**: Use `rayon` or `std::thread` to scan for multiple entity kinds in parallel on large inputs.
- **RegexSet for entities**: Use `regex::RegexSet` to match all entity patterns in a single scan instead of 9 separate regex scans.
- **SIMD string search**: Use `memchr` crate for ASCII substring search in `contains_any` and `word_contains_any`.
- **Lazy Ctx::lower**: Only lowercase the input when predicates actually need it (many predicates work on raw input).
- **Pre-compute entity regex set**: Compile all entity regexes into a single `RegexSet` at `Program` compile time.

## Memory Optimizations
- **SmallVec for triggered/explanations**: Use `smallvec` crate to avoid heap allocation for ≤32 triggered rules.
- **Box Predicate variants**: Already done for `SemanticMatch`. Consider boxing other large variants like `Vec<String>` in `ContainsAny`/`ContainsAll`.
- **Reduce Rule enum size**: The `Rule` struct contains multiple `String` fields. Consider using `&str` references for static strings.

## Cache Optimizations
- **Direct-mapped cache for RULE_TRIGGER_VEC_CACHE**: Use a fixed-size array instead of HashMap for O(1) lookup with no allocation overhead. (Tried and showed promise for repeated eval but hurt single-shot.)
- **Entry API for cache miss path**: Combine get+insert into a single `Entry::Vacant` operation to avoid double hash lookup.
- **UnsafeCell for thread-local caches**: Eliminate RefCell borrow overhead for hot-path caches. (Tried with direct-mapped cache, mixed results.)

## Compiler Optimizations
- **#[inline(always)] on evaluate_ref**: Tried and hurt performance. `#[inline]` is sufficient.
- **#[repr(u8)] on Predicate enum**: Tried and showed small improvement.
- **Sort predicates by cost**: Already done at compile time.

## Benchmark Health
- **Add compile-time benchmark**: Measure `compile_yaml` performance.
- **Add multi-threaded benchmark**: Measure performance with concurrent evaluations.
- **Add larger rule set benchmark**: Test with 100+ rules to stress the Large variant.
