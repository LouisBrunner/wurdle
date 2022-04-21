OA_GEN = openapi-generator
PACKAGES = wurdle-server
PACKAGES_FLAGS = $(addprefix -p ,$(PACKAGES))

all: generate-openapi

generate-openapi:
	@which $(OA_GEN) > /dev/null || (echo '`$(OA_GEN)` is missing, please install it separately'; false)
	$(OA_GEN) generate -i wurdle-server/api/openapi.yaml -g rust-server -o wurdle-openapi --additional-properties packageName=wurdle-openapi

check:
	cargo check --locked $(PACKAGES_FLAGS)
	cargo fmt --check $(PACKAGES_FLAGS)
	cargo clippy --no-deps $(PACKAGES_FLAGS)

.PHONY: all check generate-openapi
