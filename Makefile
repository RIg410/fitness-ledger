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
	
restart: build
	sudo docker compose up -d --build backend

logs:
	sudo docker container logs ledger-backend-1	

