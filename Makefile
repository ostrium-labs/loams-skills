SHELL := /bin/bash
.DEFAULT_GOAL := help

.PHONY: help check skills reference links template rust fmt test bootstrap sync-reference detach clean

help:
	@printf '%s\n' \
	  'make check           run every validation' \
	  'make skills          install skills for all supported agents' \
	  'make reference       verify the pinned upstream snapshot' \
	  'make links           check relative links in docs and skills' \
	  'make template        check template hygiene' \
	  'make rust            fmt + clippy + tests for starter/' \
	  'make fmt             format the Rust starter' \
	  'make test            test the Rust starter' \
	  'make bootstrap       fill company config and install skills' \
	  'make sync-reference  refresh the upstream snapshot (REF=...)' \
	  'make detach          remove the upstream snapshot and sync workflow'

check: skills-manifest reference links template rust

skills-manifest:
	python3 scripts/validate-skills.py

skills:
	scripts/install-skills.sh --target all

reference:
	scripts/check-reference.sh

links:
	python3 scripts/check-links.py

template:
	scripts/check-template.sh

rust:
	cd starter && cargo fmt --check && cargo clippy --all-targets -- -D warnings && cargo test

fmt:
	cd starter && cargo fmt

test:
	cd starter && cargo test

bootstrap:
	scripts/bootstrap-company.sh

sync-reference:
	scripts/sync-reference.sh $(if $(REF),--ref $(REF),)

detach:
	scripts/detach-upstream.sh

clean:
	cd starter && cargo clean
	rm -rf skills/manifest.json.tmp
