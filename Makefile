SHELL := /bin/bash

.PHONY: help docker-help docker-build docker-build-tools docker-up docker-up-tools docker-down docker-prod-up docker-prod-down \
	build-backend build-frontend build-orchestrator build-static build-runtime build-all build-all-with-tools \
	compose-dev-up compose-dev-tools-up compose-dev-down compose-prod-up compose-prod-down \
	runtime-tunnel-up runtime-tunnel-down runtime-tunnel-status

help:
	@echo "Targets:"
	@echo "  build-backend       Build backend-server image"
	@echo "  build-frontend      Build frontend-web image"
	@echo "  build-orchestrator  Build orchestrator image"
	@echo "  build-static        Build static-analysis image"
	@echo "  build-runtime       Build runtime-analysis image"
	@echo "  build-all           Build backend + frontend + orchestrator images"
	@echo "  build-all-with-tools Build all images including static-analysis and runtime-analysis"
	@echo "  compose-dev-up      Start dev stack (backend + frontend)"
	@echo "  compose-dev-tools-up Start dev stack with tools profile"
	@echo "  compose-dev-down    Stop dev stack"
	@echo "  compose-prod-up     Start prod compose stack"
	@echo "  compose-prod-down   Stop prod compose stack"
	@echo "  runtime-tunnel-up   Start SSH tunnel to remote Redroid"
	@echo "  runtime-tunnel-down Stop SSH tunnel to remote Redroid"
	@echo "  runtime-tunnel-status Check SSH tunnel status"
	@echo ""
	@echo "Compatibility aliases:"
	@echo "  docker-help         Show docker-specific targets"
	@echo "  docker-build        Build backend + frontend + orchestrator images"
	@echo "  docker-build-tools  Build all images including static-analysis and runtime-analysis"
	@echo "  docker-up           Start dev compose stack"
	@echo "  docker-up-tools     Start dev compose stack with tools profile"
	@echo "  docker-down         Stop dev compose stack"
	@echo "  docker-prod-up      Start prod compose stack"
	@echo "  docker-prod-down    Stop prod compose stack"

docker-help:
	$(MAKE) -C docker help

build-backend:
	$(MAKE) -C docker build-backend

build-frontend:
	$(MAKE) -C docker build-frontend

build-orchestrator:
	$(MAKE) -C docker build-orchestrator

build-static:
	$(MAKE) -C docker build-static

build-runtime:
	$(MAKE) -C docker build-runtime

build-all:
	$(MAKE) -C docker build-all

build-all-with-tools:
	$(MAKE) -C docker build-all-with-tools

compose-dev-up:
	$(MAKE) -C docker compose-dev-up

compose-dev-tools-up:
	$(MAKE) -C docker compose-dev-tools-up

compose-dev-down:
	$(MAKE) -C docker compose-dev-down

compose-prod-up:
	$(MAKE) -C docker compose-prod-up

compose-prod-down:
	$(MAKE) -C docker compose-prod-down

runtime-tunnel-up:
	$(MAKE) -C docker runtime-tunnel-up

runtime-tunnel-down:
	$(MAKE) -C docker runtime-tunnel-down

runtime-tunnel-status:
	$(MAKE) -C docker runtime-tunnel-status

docker-build:
	$(MAKE) -C docker build-all

docker-build-tools:
	$(MAKE) -C docker build-all-with-tools

docker-up:
	$(MAKE) -C docker compose-dev-up

docker-up-tools:
	$(MAKE) -C docker compose-dev-tools-up

docker-down:
	$(MAKE) -C docker compose-dev-down

docker-prod-up:
	$(MAKE) -C docker compose-prod-up

docker-prod-down:
	$(MAKE) -C docker compose-prod-down
