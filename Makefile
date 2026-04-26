build:
	@cargo build

test:
	@cargo nextest run --all-features

release:
	@cargo release commit --execute
	@cargo release tag --execute
	@cargo release push --execute

update-submodule:
	@git submodule update --init --recursive --remote

.PHONY: build test release update-submodule
