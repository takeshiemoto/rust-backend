POSTGRES_USER=todo_user
POSTGRES_PASSWORD=todo_password
POSTGRES_DB=todo_db

.PHONY: recreate-db
recreate-db: stop-db
	docker-compose rm -fv db
	docker-compose up -d db
	sleep 5
	sqlx database create --database-url=postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost/$(POSTGRES_DB)
	sqlx migrate run --database-url=postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost/$(POSTGRES_DB)

.PHONY: start-db
start-db:
	docker-compose up -d db

.PHONY: stop-db
stop-db:
	docker-compose down

.PHONY: migrate-up
migrate-up:
	sqlx migrate run --database-url=postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost/$(POSTGRES_DB)

.PHONY: migrate-down
migrate-down:
	sqlx migrate rollback --database-url=postgres://$(POSTGRES_USER):$(POSTGRES_PASSWORD)@localhost/$(POSTGRES_DB)
