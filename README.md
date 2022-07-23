# Foosball Player Database

## Building
	- get `rustup`
	- `cargo install diesel_cli --no-default-features --features "sqlite-bundled"`
	- `cargo build`

## Setting up
	- either adjust local `.env` file or set environment variables by hand, to match your preferences
	- create new sqlite DB: `diesel migration run`
	- run server app
