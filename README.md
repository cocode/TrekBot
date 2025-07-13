# Run benchmark with coverage

```
cargo run -- benchmark --program ../BasicRS/superstartrek.bas --games 5 --coverage-file coverage.json
```

# Generate reports

```
cd ../BasicRS
cargo run --bin basic_coverage -- ../TrekBot/coverage.json superstartrek.bas --verbose

cargo run --bin basic_coverage -- ../TrekBot/coverage.json superstartrek.bas --html coverage_report.html
```