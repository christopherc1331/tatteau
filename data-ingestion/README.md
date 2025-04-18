# Data Ingestion Library

A Rust library for handling data ingestion operations.

## Features

- Data ingestion from various sources
- Data validation and transformation
- Error handling and logging
- Database integration

## Usage

Add this crate to your `Cargo.toml`:

```toml
[dependencies]
data-ingestion = { path = "../data-ingestion" }
```

## Development

### Building

```bash
cargo build
```

### Testing

```bash
cargo test
```

### Environment Variables

Create a `.env` file in the project root with:

```env
API_KEY=your_api_key_here
DATABASE_URL=sqlite:tatteau.db
```

## License

This project is licensed under the MIT License. 