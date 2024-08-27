up:
	docker-compose up -d
down:
	docker-compose down

start:
	cargo leptos watch

checks: fmt test clippy test

fmt:
	cargo fmt

clippy:
	cargo clippy

test:
	cargo test
	cargo leptos end-to-end
