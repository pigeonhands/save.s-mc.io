release ?=

EXTRA_FLAGS ?= 
EXTRA_TRUNK_FLAGS ?= 
EXTRA_WORKER_FLAGS ?= 


ifdef release
	EXTRA_TRUNK_FLAGS := --release
	EXTRA_WORKER_BUILD_FLAGS := --release
else
	EXTRA_TRUNK_FLAGS := --cargo-profile dev
	EXTRA_WORKER_BUILD_FLAGS := --dev
endif

all: dev

.PHONY: clean
clean:
	rm -r ./dist || true

.PHONY: install-deps
install-deps:
	cargo install worker-build
	cargo install --locked trunk

.PHONY: build-frontend
build-frontend:
	cd ./frontend && trunk build $(EXTRA_TRUNK_FLAGS) --dist ../dist/assets

.PHONY: build-worker
build-worker:
	cd ./worker && worker-build $(EXTRA_WORKER_BUILD_FLAGS)
	mv ./worker/build ./dist

.PHONY: build
build: clean build-worker build-frontend;

.PHONY: build-release
build-release:
	$(MAKE) release=1 build

.PHONY: dev
dev:
	npx wrangler dev --env dev

.PHONY: deploy
deploy:
	URNSTILE_PRIVATE_KEY="1x0000000000000000000000000000000AA" \
		TURNSTILE_SITE_KEY="1x00000000000000000000AA" \
	npx wrangler deploy


.PHONY: get-tui-component
get-tui-component:
	cd ./frontend/static/webtui && \
		curl https://cdn.jsdelivr.net/npm/@webtui/css@0.1.2/dist/components/$(component).css \
			| sed 's/\-=/_tui=/g' \
			| sed 's/\-~/_tui~/g' \
			| grep 'tui' \
			> $(component).css

