SHELL := /bin/bash

MAKEFILE_DIR := $(patsubst %/,%,$(dir $(abspath $(lastword $(MAKEFILE_LIST)))))
ROOT_DIR := $(abspath $(MAKEFILE_DIR)/..)

PROJECT ?= nairi
VERSION ?= dev
BUILD_DATE ?= $(shell date -u +"%Y-%m-%dT%H:%M:%SZ")
COMMIT_SHA ?= $(shell git -C "$(ROOT_DIR)" rev-parse --short HEAD 2>/dev/null || echo unknown)
IMAGE_URL ?= https://github.com/REASY/nairi
IMAGE_SOURCE ?= https://github.com/REASY/nairi
IMAGE_LICENSES ?= Apache-2.0
IMAGE_AUTHORS ?= Artavazd Balaian <reasyu@gmail.com>

GHIDRA_VERSION ?= 12.0.4
GHIDRA_RELEASE_DATE ?= 20260303
APKTOOL_VERSION ?= 3.0.1

COMMON_BUILD_ARGS := \
	--build-arg VERSION="$(VERSION)" \
	--build-arg BUILD_DATE="$(BUILD_DATE)" \
	--build-arg COMMIT_SHA="$(COMMIT_SHA)" \
	--build-arg IMAGE_URL="$(IMAGE_URL)" \
	--build-arg IMAGE_SOURCE="$(IMAGE_SOURCE)" \
	--build-arg IMAGE_LICENSES="$(IMAGE_LICENSES)" \
	--build-arg IMAGE_AUTHORS="$(IMAGE_AUTHORS)"

.PHONY: help build-backend build-frontend build-orchestrator build-static build-all build-all-with-tools \
	compose-dev-up compose-dev-tools-up compose-dev-down compose-prod-up compose-prod-down

help:
	@echo "Targets:"
	@echo "  build-backend           Build backend-server image"
	@echo "  build-frontend          Build frontend-web image"
	@echo "  build-orchestrator      Build orchestrator image"
	@echo "  build-static            Build static-analysis image (requires GHIDRA_VERSION and GHIDRA_RELEASE_DATE)"
	@echo "  build-all               Build backend + frontend + orchestrator images"
	@echo "  build-all-with-tools    Build all images including static-analysis"
	@echo "  compose-dev-up          Start dev stack (backend + frontend)"
	@echo "  compose-dev-tools-up    Start dev stack with static-analysis profile"
	@echo "  compose-dev-down        Stop dev stack"
	@echo "  compose-prod-up         Start prod compose stack"
	@echo "  compose-prod-down       Stop prod compose stack"

build-backend:
	docker build \
		$(COMMON_BUILD_ARGS) \
		-f "$(MAKEFILE_DIR)/images/backend-server/Dockerfile" \
		-t $(PROJECT)/backend-server:$(VERSION) \
		"$(ROOT_DIR)"

build-frontend:
	docker build \
		$(COMMON_BUILD_ARGS) \
		-f "$(MAKEFILE_DIR)/images/frontend-web/Dockerfile" \
		-t $(PROJECT)/frontend-web:$(VERSION) \
		"$(ROOT_DIR)"

build-orchestrator:
	docker build \
		$(COMMON_BUILD_ARGS) \
		-f "$(MAKEFILE_DIR)/images/orchestrator/Dockerfile" \
		-t $(PROJECT)/orchestrator:$(VERSION) \
		"$(ROOT_DIR)"

build-static:
	@if [[ -z "$(GHIDRA_VERSION)" || -z "$(GHIDRA_RELEASE_DATE)" ]]; then \
		echo "GHIDRA_VERSION and GHIDRA_RELEASE_DATE are required."; \
		exit 1; \
	fi
	docker build \
		$(COMMON_BUILD_ARGS) \
		--build-arg APKTOOL_VERSION="$(APKTOOL_VERSION)" \
		--build-arg GHIDRA_VERSION="$(GHIDRA_VERSION)" \
		--build-arg GHIDRA_RELEASE_DATE="$(GHIDRA_RELEASE_DATE)" \
		-f "$(MAKEFILE_DIR)/images/static-analysis/Dockerfile" \
		-t $(PROJECT)/static-analysis:$(VERSION) \
		"$(ROOT_DIR)"

build-all: build-backend build-frontend build-orchestrator

build-all-with-tools: build-all build-static

compose-dev-up:
	docker compose -f "$(MAKEFILE_DIR)/compose/docker-compose.dev.yml" up --build

compose-dev-tools-up:
	@if [[ -z "$(GHIDRA_VERSION)" || -z "$(GHIDRA_RELEASE_DATE)" ]]; then \
		echo "GHIDRA_VERSION and GHIDRA_RELEASE_DATE are required."; \
		exit 1; \
	fi
	GHIDRA_VERSION="$(GHIDRA_VERSION)" \
	GHIDRA_RELEASE_DATE="$(GHIDRA_RELEASE_DATE)" \
	APKTOOL_VERSION="$(APKTOOL_VERSION)" \
	docker compose -f "$(MAKEFILE_DIR)/compose/docker-compose.dev.yml" --profile tools up --build

compose-dev-down:
	docker compose -f "$(MAKEFILE_DIR)/compose/docker-compose.dev.yml" down

compose-prod-up:
	docker compose -f "$(MAKEFILE_DIR)/compose/docker-compose.prod.yml" up -d

compose-prod-down:
	docker compose -f "$(MAKEFILE_DIR)/compose/docker-compose.prod.yml" down
