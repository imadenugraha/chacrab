SHELL := /bin/bash

BINARY := chacrab
DOCKER_COMPOSE := docker compose
POSTGRES_URL := postgres://chacrab:chacrab@localhost:5433/chacrab
MONGO_URL := mongodb://localhost:27018/chacrab

.PHONY: help check build test test-all test-backend fmt clippy run init login logout list add-password add-note update-password update-secret-notes show delete backup-export backup-import docker-up docker-down docker-logs clean

help:
	@echo "Chacrab Make targets"
	@echo ""
	@echo "Core"
	@echo "  make check           - cargo check"
	@echo "  make build           - cargo build"
	@echo "  make test            - run core integration tests"
	@echo "  make test-all        - run full test suite"
	@echo "  make fmt             - cargo fmt --all"
	@echo "  make clippy          - cargo clippy --all-targets -- -D warnings"
	@echo "  make clean           - cargo clean"
	@echo ""
	@echo "CLI (SQLite default)"
	@echo "  make run             - run app with --help"
	@echo "  make init            - initialize vault"
	@echo "  make login           - login"
	@echo "  make logout          - logout"
	@echo "  make list            - list items"
	@echo "  make add-password    - add password item"
	@echo "  make add-note        - add secure note"
	@echo "  make update-password ID=<id>|LABEL=<label>"
	@echo "  make update-secret-notes ID=<id>|LABEL=<label>"
	@echo "  make show ID=<id>    - show item by id/prefix"
	@echo "  make delete ID=<id>  - delete item by id/prefix"
	@echo "  make backup-export PATH=./vault.backup"
	@echo "  make backup-import PATH=./vault.backup"
	@echo ""
	@echo "Backend integration"
	@echo "  make docker-up       - start Postgres + Mongo"
	@echo "  make docker-down     - stop containers"
	@echo "  make docker-logs     - tail container logs"
	@echo "  make test-backend    - run backend selection test with env URLs"

check:
	cargo check

build:
	cargo build

test:
	cargo test --test backend_selection --test security_sqlite_plaintext --test sqlite_nonce_validation --test vault_service

test-all:
	cargo test

fmt:
	cargo fmt --all

clippy:
	cargo clippy --all-targets -- -D warnings

run:
	cargo run --bin $(BINARY) -- --help

init:
	cargo run --bin $(BINARY) -- init

login:
	cargo run --bin $(BINARY) -- login

logout:
	cargo run --bin $(BINARY) -- logout

list:
	cargo run --bin $(BINARY) -- list

add-password:
	cargo run --bin $(BINARY) -- add-password

add-note:
	cargo run --bin $(BINARY) -- add-note

update-password:
	@if [ -n "$(ID)" ]; then \
		cargo run --bin $(BINARY) -- update password --id "$(ID)"; \
	elif [ -n "$(LABEL)" ]; then \
		cargo run --bin $(BINARY) -- update password --label "$(LABEL)"; \
	else \
		echo "Usage: make update-password ID=<id> | LABEL=<label>"; exit 1; \
	fi

update-secret-notes:
	@if [ -n "$(ID)" ]; then \
		cargo run --bin $(BINARY) -- update secret-notes --id "$(ID)"; \
	elif [ -n "$(LABEL)" ]; then \
		cargo run --bin $(BINARY) -- update secret-notes --label "$(LABEL)"; \
	else \
		echo "Usage: make update-secret-notes ID=<id> | LABEL=<label>"; exit 1; \
	fi

show:
	@if [ -z "$(ID)" ]; then echo "Usage: make show ID=<id-or-prefix>"; exit 1; fi
	cargo run --bin $(BINARY) -- show $(ID)

delete:
	@if [ -z "$(ID)" ]; then echo "Usage: make delete ID=<id-or-prefix>"; exit 1; fi
	cargo run --bin $(BINARY) -- delete $(ID)

backup-export:
	@if [ -z "$(PATH)" ]; then echo "Usage: make backup-export PATH=./vault.backup"; exit 1; fi
	cargo run --bin $(BINARY) -- backup-export $(PATH)

backup-import:
	@if [ -z "$(PATH)" ]; then echo "Usage: make backup-import PATH=./vault.backup"; exit 1; fi
	cargo run --bin $(BINARY) -- backup-import $(PATH)

docker-up:
	$(DOCKER_COMPOSE) up -d

docker-down:
	$(DOCKER_COMPOSE) down

docker-logs:
	$(DOCKER_COMPOSE) logs -f --tail=100

test-backend:
	CHACRAB_TEST_POSTGRES_URL=$(POSTGRES_URL) \
	CHACRAB_TEST_MONGO_URL=$(MONGO_URL) \
	cargo test --test backend_selection

clean:
	cargo clean
