# include .env file and export its env vars
# (-include to ignore error if it does not exist)
-include .env

.PHONY: build clean publish test

# Variables
DOCKER_IMAGE_NAME = mgild/superior-randomness2

check_docker_env:
ifeq ($(strip $(DOCKER_IMAGE_NAME)),)
	$(error DOCKER_IMAGE_NAME is not set)
else
	@echo DOCKER_IMAGE_NAME: ${DOCKER_IMAGE_NAME}
endif

# Default make task
all: anchor_sync build

anchor_sync :; anchor keys sync
anchor_build :; anchor build
anchor_publish:; make -j 2 simple-flip-deploy callback-flip-deploy

docker_build:
	docker buildx build --platform linux/amd64 -f ./switchboard-function/Dockerfile -t ${DOCKER_IMAGE_NAME} --load ./switchboard-function
docker_publish:
	docker buildx build --platform linux/amd64 --pull -f ./switchboard-function/Dockerfile -t ${DOCKER_IMAGE_NAME} --push ./switchboard-function

build: docker_build measurement

dev: dev_docker_build measurement

publish: docker_publish measurement

measurement: check_docker_env
	@docker run -d --platform=linux/amd64 -q --name=my-switchboard-function ${DOCKER_IMAGE_NAME}:latest
	@docker cp my-switchboard-function:/measurement.txt ./measurement.txt
	@echo -n 'MrEnclve: '
	@cat measurement.txt
	@docker stop my-switchboard-function > /dev/null
	@docker rm my-switchboard-function > /dev/null

simple-flip:
	anchor run simple-flip
simple-flip-deploy:
	anchor build -p super_simple_randomness
	anchor deploy --provider.cluster devnet -p super_simple_randomness

callback-flip:
	anchor run callback-flip
callback-flip-deploy:
	anchor build -p switchboard_randomness_callback
	anchor deploy --provider.cluster devnet -p switchboard_randomness_callback

# Task to clean up the compiled rust application
clean:
	cargo clean


