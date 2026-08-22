#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

fail() {
  echo "REVALIDATION_FAIL: $*" >&2
  exit 1
}

required_root=(
  .gitignore
  LICENSE
  README.md
  AGENTS.md
  rust-toolchain.toml
)

for path in "${required_root[@]}"; do
  [[ -f "$path" ]] || fail "missing required root file: $path"
done

[[ -f docs/architecture/revalidation.md ]] || fail "missing canonical revalidation plan"
[[ -f .github/workflows/revalidation.yml ]] || fail "missing canonical revalidation workflow"

for path in \
  CLAUDE.md COPILOT.md GEMINI.md QWEN.md \
  pyproject.toml uv.lock package.json package-lock.json Makefile \
  .audiagentic .agents .qwen archive src tests; do
  [[ ! -e "$path" ]] || fail "legacy or duplicate repository surface present: $path"
done

if find . -path './.git' -prune -o -path './target' -prune -o \
  \( -name '*.pyc' -o -name '*.rs.bk' -o -name '.DS_Store' \) -print -quit | grep -q .; then
  fail "generated/local artifact committed or present in validation tree"
fi

rustc --version
cargo --version

if [[ -f Cargo.toml ]]; then
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
fi

echo "REPOSITORY_DISCIPLINE_OK"

if [[ -f Cargo.toml ]]; then
  echo "RUST_WORKSPACE_OK"
fi

if [[ -d crates/audiacore-core ]]; then
  core_manifest="crates/audiacore-core/Cargo.toml"
  core_src="crates/audiacore-core/src"

  if grep -Eq 'dependencies\]$' "$core_manifest"; then
    fail "core must have zero normal/dev/build dependencies"
  fi

  if grep -R -n -E 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment' "$core_src"; then
    fail "core contains an effect/runtime/serialization dependency surface"
  fi

  if grep -R -n -E 'CapabilityId|ComponentId|Lifecycle(State)?|Diagnostic(Code)?|ServiceRegistry|ProviderRegistry|PolicyRegistry|HostServices' "$core_src"; then
    fail "core contains vocabulary owned by a later layer"
  fi

  grep -q 'pub struct Application<C>' "$core_src/lib.rs" || fail "core lacks opaque Application<C> composition"
  grep -q 'pub struct ExecutionContext' "$core_src/lib.rs" || fail "core lacks execution/correlation identity carrier"
  grep -q 'string_id!(ApplicationId' "$core_src/lib.rs" || fail "core lacks validated application identity"
  grep -q 'string_id!(ExecutionId' "$core_src/lib.rs" || fail "core lacks validated execution identity"
  grep -q 'string_id!(CorrelationId' "$core_src/lib.rs" || fail "core lacks validated correlation identity"

  echo "CORE_LAYER_OK"
fi

# Stage 2: stable error identity is a zero-dependency foundation contract, not
# a registry/runtime. Codes must be globally unique at their definition sites.
if [[ -d crates/audiacore-errors ]]; then
  errors_manifest="crates/audiacore-errors/Cargo.toml"
  errors_src="crates/audiacore-errors/src"

  if grep -Eq 'dependencies\]$' "$errors_manifest"; then
    fail "error contract must have zero normal/dev/build dependencies"
  fi

  if grep -R -n -E 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-core' "$errors_src" "$errors_manifest"; then
    fail "error contract contains an upward/effect/runtime dependency"
  fi

  grep -q 'pub struct ErrorCode' "$errors_src/lib.rs" || fail "missing stable ErrorCode"
  grep -q 'pub enum ErrorCategory' "$errors_src/lib.rs" || fail "missing derived ErrorCategory"
  grep -q 'pub struct ErrorDefinition' "$errors_src/lib.rs" || fail "missing canonical ErrorDefinition"
  grep -q 'pub trait CodedError' "$errors_src/lib.rs" || fail "missing optional CodedError boundary trait"

  duplicate_error_codes="$(
    find crates -type f -name '*.rs' -exec grep -hoE 'ErrorCode::new\("[A-Z][A-Z0-9-]*-[0-9]{3}"\)' {} + 2>/dev/null \
      | sed -E 's/.*ErrorCode::new\("([^"]+)"\).*/\1/' \
      | sort \
      | uniq -d || true
  )"
  if [[ -n "$duplicate_error_codes" ]]; then
    echo "DUPLICATE_ERROR_CODE: each stable code must identify exactly one semantic condition" >&2
    printf '%s\n' "$duplicate_error_codes" >&2
    exit 1
  fi

  echo "ERROR_CONTRACT_OK"
fi

echo "AUDIACORE_REVALIDATION_OK"
