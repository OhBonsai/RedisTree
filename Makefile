default: help
.PHONY: default deps builder build tester test clean help

# Test if the dependencies we need to run this Makefile are installed
ifndef $(shell command -v docker)
	@echo "Docker is not available. Please install docker"
	@exit 1
endif

builder: ## build rust cross-compiler base image, for mac developers
	docker build -t ohbonsai/retree-builder -f builder.Dockerfile .


has_builder := $(shell docker images 'retree-builder' | tail -n 1 | grep 'retree-builder' | awk '{print $2}')
build: ## build retree docker image
ifndef has_builder
	@echo "Builder image is not exist, Auto make builder"
	@make builder
endif
	docker build -t ohbonsai/redistree:latest -f Dockerfile .

tester: build ## build tester image
	docker build -t retree-tester -f tester.Dockerfile .

test: tester ## do some behavioral tester
	docker run -v ${PWD}/tests:/tests --name treetest  --rm retree-tester


push: build ## push to docker hub
	docker push ohbonsai/retree-builder
	docker push ohbonsai/redistree

cluster: run-master run-slave run-sentinel ##  run sentinel cluster

# https://github.com/docker-library/redis/issues/45
# 在mac中，你无法在host直接链接进去
run-master:
	docker rm -f master || echo "no contianer is ok"
	docker run --name master --net=host -d  ohbonsai/redistree master

run-slave:
	docker rm -f slave || echo "no contianer is ok"
	docker run --name slave --net=host --env REDIS_PORT=6380 --env MASTER_IP=0.0.0.0 -d ohbonsai/redistree slave

run-sentinel:
	docker rm -f s1 || echo "no container is ok"
	docker rm -f s2 || echo "no container is ok"
	docker rm -f s3 || echo "no container is ok"
	docker run --name s1 --net=host --env REDIS_PORT=26379 --env MASTER_IP=0.0.0.0 -d ohbonsai/redistree sentinel
	docker run --name s2 --net=host --env REDIS_PORT=26380 --env MASTER_IP=0.0.0.0 -d ohbonsai/redistree sentinel
	docker run --name s3 --net=host --env REDIS_PORT=26381 --env MASTER_IP=0.0.0.0 -d ohbonsai/redistree sentinel


clean: ## clean
	rm -rf target
	rm -rf tests/.pytest_cache

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' ./Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'