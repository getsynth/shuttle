SRC_CRATES=deployer common codegen cargo-shuttle proto provisioner service
SRC=$(shell find $(SRC_CRATES) -name "*.rs" -type f -not -path "**/target/*")

COMMIT_SHA ?= $(shell git rev-parse --short HEAD)

BUILDX_CACHE?=/tmp/cache/buildx
ifeq ($(CI),true)
CACHE_FLAGS=--cache-to type=local,dest=$(BUILDX_CACHE),mode=max --cache-from type=local,src=$(BUILDX_CACHE)
endif

ifeq ($(PUSH),true)
BUILDX_OP=--push
else
BUILDX_OP=--load
endif

ifdef PLATFORMS
PLATFORM_FLAGS=--platform $(PLATFORMS)
endif

BUILDX_FLAGS=$(BUILDX_OP) $(PLATFORM_FLAGS) $(CACHE_FLAGS)

TAG?=$(shell git describe --tags)

DOCKER?=docker

DOCKER_COMPOSE=$(shell which docker-compose)
ifeq ($(DOCKER_COMPOSE),)
DOCKER_COMPOSE=$(DOCKER) compose
endif

POSTGRES_PASSWORD?=postgres
MONGO_INITDB_ROOT_USERNAME?=mongodb
MONGO_INITDB_ROOT_PASSWORD?=password

ifeq ($(PROD),true)
DOCKER_COMPOSE_FILES=-f docker-compose.yml
STACK=shuttle-prod
APPS_FQDN=shuttleapp.rs
DB_FQDN=db.shuttle.rs
CONTAINER_REGISTRY=public.ecr.aws/shuttle
else
DOCKER_COMPOSE_FILES=-f docker-compose.yml -f docker-compose.dev.yml
STACK=shuttle-dev
APPS_FQDN=unstable.shuttleapp.rs
DB_FQDN=db.unstable.shuttle.rs
CONTAINER_REGISTRY=public.ecr.aws/shuttle-dev
endif

POSTGRES_EXTRA_PATH?=./extras/postgres
POSTGRES_TAG?=14

RUST_LOG?=debug

DOCKER_COMPOSE_ENV=STACK=$(STACK) BACKEND_TAG=$(TAG) PROVISIONER_TAG=$(TAG) POSTGRES_TAG=${POSTGRES_TAG} APPS_FQDN=$(APPS_FQDN) DB_FQDN=$(DB_FQDN) POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) RUST_LOG=$(RUST_LOG) CONTAINER_REGISTRY=$(CONTAINER_REGISTRY) MONGO_INITDB_ROOT_USERNAME=$(MONGO_INITDB_ROOT_USERNAME) MONGO_INITDB_ROOT_PASSWORD=$(MONGO_INITDB_ROOT_PASSWORD)

.PHONY: images clean src up down deploy shuttle-% postgres docker-compose.rendered.yml test bump-% --validate-version

clean:
	rm .shuttle-*
	rm docker-compose.rendered.yml

images: shuttle-provisioner shuttle-deployer shuttle-gateway postgres

postgres:
	docker buildx build \
	       --build-arg POSTGRES_TAG=$(POSTGRES_TAG) \
	       --tag $(CONTAINER_REGISTRY)/postgres:$(POSTGRES_TAG) \
	       $(BUILDX_FLAGS) \
	       -f $(POSTGRES_EXTRA_PATH)/Containerfile \
	       $(POSTGRES_EXTRA_PATH)

docker-compose.rendered.yml: docker-compose.yml docker-compose.dev.yml
	$(DOCKER_COMPOSE_ENV) $(DOCKER_COMPOSE) $(DOCKER_COMPOSE_FILES) -p $(STACK) config > $@

deploy: docker-compose.yml
	$(DOCKER_COMPOSE_ENV) docker stack deploy -c $< $(STACK)

test:
	cd e2e; POSTGRES_PASSWORD=$(POSTGRES_PASSWORD) APPS_FQDN=$(APPS_FQDN) cargo test $(CARGO_TEST_FLAGS) -- --nocapture

up: docker-compose.rendered.yml images
	CONTAINER_REGISTRY=$(CONTAINER_REGISTRY) $(DOCKER_COMPOSE) -f $< -p $(STACK) up -d

down: docker-compose.rendered.yml
	CONTAINER_REGISTRY=$(CONTAINER_REGISTRY) $(DOCKER_COMPOSE) -f $< -p $(STACK) down

shuttle-%: ${SRC} Cargo.lock
	docker buildx build \
	       --build-arg crate=shuttle-$(*) \
	       --tag $(CONTAINER_REGISTRY)/$(*):$(COMMIT_SHA) \
	       --tag $(CONTAINER_REGISTRY)/$(*):$(TAG) \
	       --tag $(CONTAINER_REGISTRY)/$(*):latest \
	       $(BUILDX_FLAGS) \
	       -f Containerfile \
	       .

# Bunch of targets to make bumping the shuttle version easier
bump-version: --validate-version
	git checkout -b "chore/v$(version)"
	cargo set-version --workspace "$(version)"

	$(call next, bump-resources)

bump-resources:
	git commit -am "chore: v$(version)"
	fastmod --fixed-strings $(current) $(version) resources

	$(call next, bump-examples)

bump-examples:
	git commit -am "chore: resources v$(version)"
	fastmod --fixed-strings $(current) $(version) examples

	$(call next, bump-misc)

bump-misc:
	git commit -am "docs: v$(version)"
	fastmod --fixed-strings $(current) $(version)

	$(call next, bump-final)

bump-final:
	git commit -am "misc: v$(version)"

define next
	git add --all
	git --no-pager diff --staged

	echo -e "\x1B[36m>> Is this correct?\x1B[39m"
	read yn; if [ $$yn != "y" ]; then echo "Fix the issues then continue with:"; echo "make version=$(version) current=$(current) $1"; exit 2; fi

	make $1
endef

--validate-version:
	echo "$(version)" | rg -q "\d+\.\d+\.\d+" || { echo "version argument must be in the form x.y.z"; exit 1; }
	echo "$(current)" | rg -q "\d+\.\d+\.\d+" || { echo "current argument must be in the form x.y.z"; exit 1; }
