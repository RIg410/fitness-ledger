up:
	docker-compose up -d

fmt:
	cd back && cargo fmt
	cd front && cargo fmt
