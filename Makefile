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

restart-nginx: 
	sudo docker image prune -a -f
	sudo docker compose build nginx
	sudo docker compose up -d --build nginx

restart-back: 
	cargo build --release	
	sudo docker image prune -a -f
	sudo docker compose build backend
	sudo docker compose up -d --build backend

restart: restart-nginx restart-back

logs:
	sudo docker container logs ledger-backend-1	


