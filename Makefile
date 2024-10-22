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
	sudo docker image prune -a -f
	sudo docker compose build backend
	sudo docker compose build nginx

	
restart-back: build
	sudo docker compose up -d --build backend

restart-nginx: build
	sudo docker compose up -d --build nginx

restart-back: build
	sudo docker compose up -d --build backend

restart: build
	sudo docker compose up -d --build nginx
	sudo docker compose up -d --build backend

logs:
	sudo docker container logs ledger-backend-1	

