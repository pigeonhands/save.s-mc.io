
.PHONY: clean
clean:
	rm -r ./dist || true

.PHONY: install-deps
install-deps:
	cargo install worker-build
	cargo install --locked trunk

.PHONY: build-worker
build-worker:
	cd ./worker && worker-build --release
	mv ./worker/build ./dist

.PHONY: build-frontend
build-frontend:
	cd ./frontend && trunk build --dist ../dist/assets

.PHONY: build
build: clean build-worker build-frontend;

.PHONY: dev
dev: build
	npx wrangler dev

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

