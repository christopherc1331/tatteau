# Tatteau

A workspace containing two crates:
- `data-ingestion`: Data ingestion and processing
- `web`: Web application built with Leptos

## Building and Testing

### Building All Crates

To build all crates in the workspace:
```bash
cargo build
```

### Building Individual Crates

#### Data Ingestion Crate
```bash
cargo build -p data-ingestion
```

#### Web Crate
The web crate requires both SSR and hydration features:

```bash
cargo build -p web --features "ssr hydrate"
```

### Running Tests

To run tests for all crates:
```bash
cargo test
```

To run tests for a specific crate:

#### Data Ingestion Crate
```bash
cargo test -p data-ingestion
```

#### Web Crate
```bash
cargo test -p web --features "ssr hydrate"
```

### Development

For development, you can use `cargo check` instead of `cargo build` for faster compilation:

```bash
# Check all crates
cargo check

# Check data-ingestion crate
cargo check -p data-ingestion

# Check web crate
cargo check -p web --features "ssr hydrate"
``` 