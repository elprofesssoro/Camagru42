COMPOSE = docker-compose

all: up

up:
	$(COMPOSE) up -d --build
build:
	$(COMPOSE) build
down:
	$(COMPOSE) down
stop:
	$(COMPOSE) stop

rebuild: down up
build-frontend:
	$(COMPOSE) build frontend
rebuild-frontend:
	$(COMPOSE) up -d --build frontend
build-backend:
	$(COMPOSE) build backend
rebuild-backend:
	$(COMPOSE) up -d --build backend
build-db:
	$(COMPOSE) build db
rebuild-db:
	$(COMPOSE) up -d --build db

logs:
	$(COMPOSE) logs -f

clean:
	$(COMPOSE) down -v --rmi all

.PHONY: all up build down stop rebuild build-frontend rebuild-frontend build-backend rebuild-backend build-db rebuild-db logs clean
