.PHONY: help start stop restart logs logs-backend logs-frontend logs-db build test clean dev-backend dev-frontend db-connect db-reset

help: ## Show this help message
	@echo 'Usage: make [target]'
	@echo ''
	@echo 'Available targets:'
	@awk 'BEGIN {FS = ":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  %-20s %s\n", $$1, $$2}' $(MAKEFILE_LIST)

start: ## Start all services with docker-compose
	docker-compose up -d

stop: ## Stop all services
	docker-compose down

restart: ## Restart all services
	docker-compose restart

logs: ## View logs from all services
	docker-compose logs -f

logs-backend: ## View backend logs
	docker-compose logs -f backend

logs-frontend: ## View frontend logs
	docker-compose logs -f frontend

logs-db: ## View database logs
	docker-compose logs -f postgres

build: ## Build all services
	docker-compose build

rebuild: ## Rebuild and restart all services
	docker-compose up -d --build

test: ## Run API tests
	./test.sh

clean: ## Remove all containers and volumes
	docker-compose down -v

dev-backend: ## Run backend in development mode
	cd rugistry-service && cargo run

dev-frontend: ## Run frontend in development mode
	cd rugistry-frontend && npm run dev

db-connect: ## Connect to PostgreSQL database
	docker-compose exec postgres psql -U rugistry_user -d rugistry

db-reset: ## Reset database (WARNING: deletes all data)
	docker-compose down -v
	docker-compose up -d postgres
	@echo "Waiting for postgres to be ready..."
	@sleep 5
	docker-compose up -d backend

check-backend: ## Check backend code without building
	cd rugistry-service && cargo check

format-backend: ## Format backend code
	cd rugistry-service && cargo fmt

lint-backend: ## Lint backend code
	cd rugistry-service && cargo clippy

format-frontend: ## Format frontend code
	cd rugistry-frontend && npm run format || echo "Add format script to package.json"

ps: ## Show running containers
	docker-compose ps

status: ## Show service status and URLs
	@echo "=== Rugistry Service Status ==="
	@echo ""
	@docker-compose ps
	@echo ""
	@echo "=== Service URLs ==="
	@echo "Frontend:  http://localhost:5173"
	@echo "Backend:   http://localhost:3000"
	@echo "Health:    http://localhost:3000/health"
	@echo "PostgreSQL: localhost:5432"
	@echo ""
	@echo "Run 'make logs' to view logs"
	@echo "Run 'make test' to run tests"
