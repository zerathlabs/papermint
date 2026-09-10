.PHONY: help test check clippy doc fmt bench bench-check clean example docs-dev docs-build bump

help:
	@echo "papermint — High-performance, type-safe thermal receipt printing"
	@echo ""
	@echo "Available make commands:"
	@echo "  make test        - Run all unit and integration tests across workspace"
	@echo "  make check       - Fast compile & typecheck with all features across workspace"
	@echo "  make clippy      - Run strict Clippy linter with -D warnings on all targets"
	@echo "  make doc         - Build documentation (cargo doc --no-deps)"
	@echo "  make fmt         - Format all Rust source code (cargo fmt)"
	@echo "  make bench       - Run Criterion performance microbenchmarks"
	@echo "  make bench-check - Fast sanity check of benchmark harness"
	@echo "  make example     - Run the restaurant bill example"
	@echo "  make docs-dev    - Start local Astro Starlight docs development server"
	@echo "  make docs-build  - Build production static documentation website"
	@echo "  make bump V=x.y.z - Bump version across all Cargo.toml, package.json & lockfiles"
	@echo "  make clean       - Clean target build artifacts"

test:
	cargo test --workspace --all-features

check:
	cargo check --workspace --all-features

clippy:
	cargo clippy --workspace --all-targets --all-features -- -D warnings

doc:
	cargo doc --workspace --all-features --no-deps

fmt:
	cargo fmt

bench:
	cargo bench --bench receipt_benchmarks

bench-check:
	cargo bench --bench receipt_benchmarks -- --test

example:
	cargo run --example restaurant_bill

docs-dev:
	cd web && pnpm dev

docs-build:
	cd web && pnpm run build

clean:
	cargo clean

bump:
	@if [ -z "$(V)" ]; then echo "❌ Error: Version required. Example: make bump V=0.1.1"; exit 1; fi
	@echo "Bumping all packages to $(V)..."
	@node -e "\
		const fs = require('fs'); \
		const v = '$(V)'; \
		const tomls = ['Cargo.toml', 'crates/papermint-daemon/Cargo.toml', 'crates/papermint-mobile/Cargo.toml', 'crates/papermint-node/Cargo.toml']; \
		tomls.forEach(f => { \
			let c = fs.readFileSync(f, 'utf8'); \
			c = c.replace(/^version = \".*?\"/m, 'version = \"' + v + '\"'); \
			fs.writeFileSync(f, c, 'utf8'); \
		}); \
		const pkgPath = 'crates/papermint-node/package.json'; \
		let pkg = JSON.parse(fs.readFileSync(pkgPath, 'utf8')); \
		pkg.version = v; \
		fs.writeFileSync(pkgPath, JSON.stringify(pkg, null, 2) + '\n', 'utf8'); \
		const idxPath = 'crates/papermint-node/index.js'; \
		let idx = fs.readFileSync(idxPath, 'utf8'); \
		idx = idx.replace(/expected \\d+\\.\\d+\\.\\d+/g, 'expected ' + v).replace(/!== '\\d+\\.\\d+\\.\\d+'/g, '!== \'' + v + '\''); \
		fs.writeFileSync(idxPath, idx, 'utf8'); \
	"
	@cargo check --workspace --quiet
	@echo "✅ All packages bumped to $(V) and Cargo.lock synced!"
	@echo ""
	@echo "To release, commit and tag:"
	@echo "  git add ."
	@echo "  git commit -m \"chore: release v$(V)\""
	@echo "  git tag v$(V)"
	@echo "  git push && git push origin v$(V)"

