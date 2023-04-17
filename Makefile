include .env

.PHONY: recreate-db
recreate-db: stop-db
	docker-compose rm -fv db
	docker-compose up -d db
	sleep 5
	sqlx database create --database-url=$(DATABASE_URL)
	sqlx migrate run --database-url=$(DATABASE_URL)

.PHONY: start-db
start-db:
	docker-compose up -d db

.PHONY: stop-db
stop-db:
	docker-compose down

.PHONY: migrate-up
migrate-up:
	sqlx migrate run --database-url=$(DATABASE_URL)

.PHONY: migrate-down
migrate-down:
	sqlx migrate rollback --database-url=$(DATABASE_URL)

.PHONY: setup-db
setup-db:
	docker-compose up -d db
	sleep 5
	sqlx migrate run --database-url=$(DATABASE_URL)
