up:
	docker compose up -d --build
down:
	docker compose down

start:
	cargo leptos watch

checks: fmt test clippy test

fmt:
	cargo fmt

clippy:
	cargo clippy

test:
	cargo test

build:
	cargo build --release	
	sudo docker compose build
