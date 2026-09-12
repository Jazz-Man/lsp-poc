# Single source of verification commands — the contract referenced by
# CLAUDE.md and .claude/rules/tech.md. Edit freely; keep targets .PHONY and
# bodies plain.

# Route cargo through rtk locally (token-optimized output, transparent
# pass-through for commands rtk does not filter); plain cargo under CI, where
# CI=true is set by the runner and rtk is not installed.
ifeq ($(CI),)
CARGO_BIN := rtk cargo
else
CARGO_BIN := cargo
endif

# Toolchains (dated pins; upgrades are deliberate, never casual).
MIRI_TOOLCHAIN ?= nightly-2026-09-08

# Miri cross-interprets for x86_64: the document-parsing tests this repo is
# heading toward exercise ropey/str_indices NEON paths the interpreter cannot
# run on an aarch64 host (fork-inherited rationale).
TARGET := x86_64-apple-darwin

.DEFAULT_GOAL := help
.PHONY: help fmt fmt-fix clippy doc test dylint battery dupes deny mutants miri miri-setup

## help: list available targets
help:
	@rtk grep -E '^## ' $(MAKEFILE_LIST) | sed 's/^## //'

## fmt: check formatting (gate)
fmt:
	@$(CARGO_BIN) fmt --all --check

## fmt-fix: apply formatting
fmt-fix:
	@$(CARGO_BIN) fmt --all

## clippy: lint every target, warnings as errors (gate)
clippy:
	@$(CARGO_BIN) clippy --workspace --all-targets -- -D warnings

## doc: build docs, warnings as errors (gate)
doc:
	@RUSTDOCFLAGS="--enable-index-page -Zunstable-options -D warnings" $(CARGO_BIN) +nightly doc --workspace --no-deps

## test: nextest run (gate)
# No doctest leg: cargo test --doc errors on this workspace today (lsp-poc is
# bin-only, zed-lsp-poc is a cdylib — no library targets to run doctests for).
# Add the leg back together with the first [lib] target, as in the fork.
test:
	@$(CARGO_BIN) nextest run --workspace

## dylint: external lint suites (gate)
dylint:
	@$(CARGO_BIN) dylint --all -- --all-targets

## battery: the full pre-done gate
battery: fmt clippy doc test dylint

## dupes: duplication check (on demand)
dupes:
	@$(CARGO_BIN) dupes check

## deny: dependency policies (on demand; findings gate)
deny:
	@$(CARGO_BIN) deny check

# Diagnostic sweep, never a gate; the battery never runs it. Heavy: run it
# alone — a concurrent build poisons its auto-derived per-scenario timeout.
# Exit code 2 means survivors were found. FILE=crates/lsp-poc/src/foo.rs
# scopes the sweep to one file.
## mutants: mutation-testing sweep (diagnostic, never a gate)
mutants:
	@$(CARGO_BIN) mutants $(if $(FILE),-f $(FILE))

## miri: UB interpreter over the tests (on demand, slow)
# Never in the battery; no --no-default-features leg (no [features] yet —
# added with the first feature).
# architecture_rules_hold is excluded by name: its workspace-wide fs walk does
# not terminate under the interpreter (fork observed >1h) — infeasible, not failing.
# That exclusion empties the run today — the workspace's only test is the
# excluded one — and nextest's --no-tests default (auto) treats zero matches
# as an error. --no-tests=warn states the real contract: an empty run is a
# visible success, not a failure. Canary: if this warning ever fires, either
# every test became miri-excluded or the -E filter stopped matching — check
# the exclusion name before dismissing it.
miri:
	@$(CARGO_BIN) +$(MIRI_TOOLCHAIN) miri nextest run --target $(TARGET) --no-tests=warn -E 'not(test(architecture_rules_hold))'

## miri-setup: one-time toolchain preparation for make miri
miri-setup:
	@rustup component add miri --toolchain $(MIRI_TOOLCHAIN)
	@rustup target add $(TARGET) --toolchain $(MIRI_TOOLCHAIN)
	@$(CARGO_BIN) +$(MIRI_TOOLCHAIN) miri setup --target $(TARGET)
