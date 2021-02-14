default: help
.PHONY: default deps builder build tester test clean help

# Test if the dependencies we need to run this Makefile are installed
ifndef $(shell command -v docker)
	@echo "Docker is not available. Please install docker"
	@exit 1
endif

builder: ## build rust cross-compiler base image, for mac developers
	docker build -t retree-builder -f builder.Dockerfile .


has_builder := $(shell docker images 'retree-builder' | tail -n 1 | grep 'retree-builder' | awk '{print $2}')
build: ## build retree docker image
ifndef has_builder
	@echo "Builder image is not exist, Auto make builder"
	@make builder
endif
	docker build -t retree -f Dockerfile .

tester: build ## build tester image
	docker build -t retree-tester -f tester.Dockerfile .

test: tester ## do some behavioral tester
	docker run -v ${PWD}/tests:/tests --name treetest  --rm retree-tester 

clean: ## clean
	rm -rf target
	rm -rf tests/.pytest_cache

help:
	@grep -E '^[a-zA-Z_-]+:.*?## .*$$' ./Makefile | sort | awk 'BEGIN {FS = ":.*?## "}; {printf "\033[36m%-30s\033[0m %s\n", $$1, $$2}'