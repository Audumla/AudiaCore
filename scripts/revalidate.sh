#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

fail() {
  echo "REVALIDATION_FAIL: $*" >&2
  exit 1
}

normal_dependencies() {
  awk '
    /^\[dependencies\]$/ { in_deps=1; next }
    /^\[/ { in_deps=0 }
    in_deps && /^[A-Za-z0-9_-]+[[:space:]]*=/ {
      key=$1; sub(/[[:space:]]*=.*$/, "", key); print key
    }
  ' "$1"
}

assert_only_dependency() {
  manifest="$1"
  expected="$2"
  deps="$(normal_dependencies "$manifest")"
  [[ "$deps" == "$expected" ]] || fail "$manifest normal dependencies must be exactly: $expected (found: ${deps:-none})"
  if grep -Eq '^\[(dev-dependencies|build-dependencies)\]$' "$manifest"; then
    fail "$manifest must not add dev/build dependencies at this layer"
  fi
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

for crate in sensitive template reconcile; do
  crate_dir="crates/audiacore-$crate"
  if [[ -d "$crate_dir" ]]; then
    assert_only_dependency "$crate_dir/Cargo.toml" "audiacore-errors"

    if grep -R -n -E 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-core|audiacore-host' "$crate_dir"; then
      fail "$crate must remain a pure foundation semantic crate"
    fi

    grep -q 'impl CodedError for' "$crate_dir/src/lib.rs" || fail "$crate public semantic failures must have stable coded identity"
  fi
done

if [[ -d crates/audiacore-sensitive && -d crates/audiacore-template && -d crates/audiacore-reconcile ]]; then
  grep -q 'pub struct Sensitive<T>' crates/audiacore-sensitive/src/lib.rs || fail "sensitive crate lacks explicit Sensitive<T> wrapper"
  grep -q 'pub struct Template' crates/audiacore-template/src/lib.rs || fail "template crate lacks deterministic Template"
  grep -q 'pub enum ReconcileAction<T>' crates/audiacore-reconcile/src/lib.rs || fail "reconcile crate lacks effect-as-data plan"
  echo "PURE_FOUNDATION_PRIMITIVES_OK"
fi

if [[ -d crates/audiacore-config ]]; then
  config_manifest="crates/audiacore-config/Cargo.toml"
  config_src="crates/audiacore-config/src/lib.rs"
  config_deps="$(normal_dependencies "$config_manifest")"
  expected_config_deps="$(printf '%s\n' audiacore-errors serde toml)"
  [[ "$config_deps" == "$expected_config_deps" ]] || fail "config dependencies must be exactly audiacore-errors, serde, toml"

  grep -Fq 'toml = { version = "1.1.4", default-features = false, features = ["std", "serde", "parse"] }' "$config_manifest" || fail "config must use TOML without provider/display extras"
  grep -Fq 'serde = { version = "1.0.229", features = ["derive"] }' "$config_manifest" || fail "config tests must isolate Serde derive to dev dependency"

  if grep -R -n -E 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-core|audiacore-host|Policy|from_env|from_file|read_to_string' crates/audiacore-config; then
    fail "config contains acquisition, effect, policy, runtime, or upward-layer semantics"
  fi

  grep -q 'pub struct ConfigLayerId' "$config_src" || fail "config lacks typed layer identity"
  grep -q 'pub struct ConfigRevision' "$config_src" || fail "config lacks provenance revision"
  grep -q 'pub struct ResolvedConfig<T>' "$config_src" || fail "config lacks provenance-carrying resolved value"
  grep -q 'pub struct ConfigLayers' "$config_src" || fail "config lacks explicit ordered layer composition"
  grep -q 'pub fn merge_toml' "$config_src" || fail "config lacks explicit in-memory TOML merge"
  grep -q 'impl CodedError for ConfigError' "$config_src" || fail "config failures lack stable coded identity"

  if grep -q 'into_value' "$config_src"; then
    fail "config must not provide a convenience API that silently discards provenance"
  fi

  echo "CONFIG_FOUNDATION_OK"
fi

if [[ -d crates/audiacore-host ]]; then
  host_manifest="crates/audiacore-host/Cargo.toml"
  host_src="crates/audiacore-host/src/lib.rs"
  host_deps="$(normal_dependencies "$host_manifest")"
  expected_host_deps="$(printf '%s\n' audiacore-errors audiacore-sensitive)"
  [[ "$host_deps" == "$expected_host_deps" ]] || fail "host dependencies must be exactly audiacore-errors and audiacore-sensitive"

  if grep -Eq '^\[(dev-dependencies|build-dependencies)\]$' "$host_manifest"; then
    fail "host contract must not add dev/build dependencies"
  fi

  if grep -R -n -E 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-config|audiacore-core' crates/audiacore-host; then
    fail "host contract contains native effects, runtime, config, or core coupling"
  fi

  grep -q 'pub struct FileReadAuthority' "$host_src" || fail "host lacks explicit file read authority"
  grep -q 'pub struct FileWriteAuthority' "$host_src" || fail "host lacks explicit file write authority"
  grep -q 'pub trait FileHost' "$host_src" || fail "host lacks file effect contract"
  grep -q 'fn read_optional' "$host_src" || fail "file host lacks optional observation operation"
  grep -q 'fn write' "$host_src" || fail "file host lacks write operation"
  grep -q 'fn remove' "$host_src" || fail "file host lacks remove operation"
  grep -q 'impl CodedError for FileAuthorityError' "$host_src" || fail "file authority validation lacks stable coded identity"

  if grep -q 'fn read(' "$host_src"; then
    fail "mandatory file read is not yet justified by a consumer"
  fi
  if grep -Eq 'fn[[:space:]]+allows[[:space:]]*\(' "$host_src"; then
    fail "host contract must not pretend lexical path checks prove safe containment"
  fi

  grep -q 'pub struct ProcessAuthority' "$host_src" || fail "host lacks explicit process launch authority"
  grep -q 'pub struct ProcessRequest' "$host_src" || fail "host lacks explicit process request"
  grep -q 'pub enum ProcessStdio' "$host_src" || fail "host lacks explicit stdio ownership mode"
  grep -q 'pub struct ProcessExit' "$host_src" || fail "host lacks neutral process exit value"
  grep -q 'pub trait ProcessChild' "$host_src" || fail "host lacks owned process child lifecycle"
  grep -q 'pub trait ProcessHost' "$host_src" || fail "host lacks process spawn contract"
  grep -q 'fn take_stdin' "$host_src" || fail "process child lacks owned stdin transfer"
  grep -q 'fn take_stdout' "$host_src" || fail "process child lacks owned stdout transfer"
  grep -q 'fn take_stderr' "$host_src" || fail "process child lacks owned stderr transfer"
  grep -q 'fn try_wait' "$host_src" || fail "process child lacks nonblocking lifecycle observation"
  grep -q 'fn wait' "$host_src" || fail "process child lacks wait lifecycle operation"
  grep -q 'fn kill' "$host_src" || fail "process child lacks direct-child termination operation"
  grep -q 'fn spawn' "$host_src" || fail "process host lacks spawn operation"
  grep -q 'Sensitive<OsString>' "$host_src" || fail "process environment values must be sensitive by construction"
  grep -q 'inherit_environment: false' "$host_src" || fail "ambient environment inheritance must default off"
  grep -q 'impl CodedError for ProcessContractError' "$host_src" || fail "process contract validation lacks stable coded identity"

  if grep -Eq 'fn[[:space:]]+(stdin|stdout|stderr)[[:space:]]*\(&mut self\)' "$host_src"; then
    fail "borrowed process stdio access is not justified; keep the smaller owned take_* contract"
  fi
  if grep -Eq 'fn[[:space:]]+run[[:space:]]*\(' "$host_src"; then
    fail "one-shot process run would collapse the owned child lifecycle"
  fi
  if grep -Eq 'File(Store|Service|Manager)|HostServices|HostRegistry|ProcessManager|ProcessRegistry|NetworkHost|SecretHost|HostFuture' "$host_src"; then
    fail "host contains an unproven service, manager, registry, facility, or async abstraction"
  fi

  echo "FILE_HOST_CONTRACT_OK"
  echo "PROCESS_HOST_CONTRACT_OK"
fi

echo "AUDIACORE_REVALIDATION_OK"
