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

assert_dependencies() {
  local manifest="$1"
  local expected="$2"
  local actual
  actual="$(normal_dependencies "$manifest")"
  [[ "$actual" == "$expected" ]] || fail "$manifest dependencies differ; expected [$expected], found [${actual:-none}]"
}

assert_no_dev_or_build_dependencies() {
  local manifest="$1"
  if grep -Eq '^\[(dev-dependencies|build-dependencies)\]$' "$manifest"; then
    fail "$manifest must not add dev/build dependencies at this layer"
  fi
}

assert_no_match() {
  local pattern="$1"
  local message="$2"
  shift 2
  if grep -R -n -E "$pattern" "$@"; then
    fail "$message"
  fi
}

for path in .gitignore LICENSE README.md AGENTS.md rust-toolchain.toml \
  docs/architecture/revalidation.md .github/workflows/revalidation.yml; do
  [[ -f "$path" ]] || fail "missing required repository file: $path"
done
for path in CLAUDE.md COPILOT.md GEMINI.md QWEN.md pyproject.toml uv.lock \
  package.json package-lock.json Makefile .audiagentic .agents .qwen archive src tests; do
  [[ ! -e "$path" ]] || fail "legacy or duplicate repository surface present: $path"
done
if find . -path './.git' -prune -o -path './target' -prune -o \
  \( -name '*.pyc' -o -name '*.rs.bk' -o -name '.DS_Store' \) -print -quit | grep -q .; then
  fail "generated/local artifact present in validation tree"
fi

rustc --version
cargo --version
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo test --workspace --locked

echo "REPOSITORY_DISCIPLINE_OK"
echo "RUST_WORKSPACE_OK"

core_manifest="crates/audiacore-core/Cargo.toml"
core_src="crates/audiacore-core/src"
[[ -f "$core_manifest" ]] || fail "core crate missing"
if grep -Eq 'dependencies\]$' "$core_manifest"; then
  fail "core must have zero normal/dev/build dependencies"
fi
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment' \
  "core contains effect/runtime/serialization semantics" "$core_src"
assert_no_match 'CapabilityId|ComponentId|Lifecycle(State)?|Diagnostic(Code)?|ServiceRegistry|ProviderRegistry|PolicyRegistry|HostServices' \
  "core contains vocabulary owned by a later layer" "$core_src"
grep -q 'pub struct Application<C>' "$core_src/lib.rs" || fail "core lacks Application<C>"
grep -q 'pub struct ExecutionContext' "$core_src/lib.rs" || fail "core lacks ExecutionContext"
grep -q 'string_id!(ApplicationId' "$core_src/lib.rs" || fail "core lacks ApplicationId"
grep -q 'string_id!(ExecutionId' "$core_src/lib.rs" || fail "core lacks ExecutionId"
grep -q 'string_id!(CorrelationId' "$core_src/lib.rs" || fail "core lacks CorrelationId"
echo "CORE_LAYER_OK"

errors_manifest="crates/audiacore-errors/Cargo.toml"
errors_src="crates/audiacore-errors/src"
[[ -f "$errors_manifest" ]] || fail "stable error crate missing"
if grep -Eq 'dependencies\]$' "$errors_manifest"; then
  fail "stable error contract must have zero normal/dev/build dependencies"
fi
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-core' \
  "stable error contract contains upward/effect/runtime semantics" "$errors_src" "$errors_manifest"
grep -q 'pub struct ErrorCode' "$errors_src/lib.rs" || fail "missing ErrorCode"
grep -q 'pub enum ErrorCategory' "$errors_src/lib.rs" || fail "missing ErrorCategory"
grep -q 'pub struct ErrorDefinition' "$errors_src/lib.rs" || fail "missing ErrorDefinition"
grep -q 'pub trait CodedError' "$errors_src/lib.rs" || fail "missing CodedError"
duplicate_error_codes="$(
  find crates -type f -name '*.rs' -exec grep -hoE 'ErrorCode::new\("[A-Z][A-Z0-9-]*-[0-9]{3}"\)' {} + 2>/dev/null \
    | sed -E 's/.*ErrorCode::new\("([^"]+)"\).*/\1/' | sort | uniq -d || true
)"
[[ -z "$duplicate_error_codes" ]] || fail "duplicate stable error codes: $duplicate_error_codes"
echo "ERROR_CONTRACT_OK"

for crate in sensitive template reconcile; do
  dir="crates/audiacore-$crate"
  assert_dependencies "$dir/Cargo.toml" "audiacore-errors"
  assert_no_dev_or_build_dependencies "$dir/Cargo.toml"
  assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-core|audiacore-host' \
    "$crate must remain pure foundation semantics" "$dir"
  grep -q 'impl CodedError for' "$dir/src/lib.rs" || fail "$crate failures need stable coded identity"
done
grep -q 'pub struct Sensitive<T>' crates/audiacore-sensitive/src/lib.rs || fail "Sensitive<T> missing"
grep -q 'pub struct Template' crates/audiacore-template/src/lib.rs || fail "Template missing"
grep -q 'pub enum ReconcileAction<T>' crates/audiacore-reconcile/src/lib.rs || fail "reconciliation effect-as-data missing"
echo "PURE_FOUNDATION_PRIMITIVES_OK"

config_manifest="crates/audiacore-config/Cargo.toml"
config_src="crates/audiacore-config/src/lib.rs"
expected_config_deps="$(printf '%s\n' audiacore-errors serde toml)"
assert_dependencies "$config_manifest" "$expected_config_deps"
grep -Fq 'toml = { version = "1.1.4", default-features = false, features = ["std", "serde", "parse"] }' "$config_manifest" \
  || fail "config TOML feature surface changed"
grep -Fq 'serde = { version = "1.0.229", features = ["derive"] }' "$config_manifest" \
  || fail "config test derive dependency changed"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-core|audiacore-host|Policy|from_env|from_file|read_to_string' \
  "config acquired effects/provider/policy/upward semantics" crates/audiacore-config
grep -q 'pub struct ConfigLayerId' "$config_src" || fail "ConfigLayerId missing"
grep -q 'pub struct ConfigRevision' "$config_src" || fail "ConfigRevision missing"
grep -q 'pub struct ResolvedConfig<T>' "$config_src" || fail "ResolvedConfig<T> missing"
grep -q 'pub struct ConfigLayers' "$config_src" || fail "ConfigLayers missing"
grep -q 'pub fn merge_toml' "$config_src" || fail "in-memory TOML merge missing"
grep -q 'impl CodedError for ConfigError' "$config_src" || fail "config coded errors missing"
! grep -q 'into_value' "$config_src" || fail "config must not silently discard provenance"
echo "CONFIG_FOUNDATION_OK"
echo "PURE_FOUNDATION_OK"

host_manifest="crates/audiacore-host/Cargo.toml"
host_src="crates/audiacore-host/src/lib.rs"
expected_host_deps="$(printf '%s\n' audiacore-errors audiacore-sensitive)"
assert_dependencies "$host_manifest" "$expected_host_deps"
assert_no_dev_or_build_dependencies "$host_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-config|audiacore-core' \
  "host contract contains native effects/runtime/config/core coupling" crates/audiacore-host
for symbol in FileReadAuthority FileWriteAuthority FileHost ProcessAuthority ProcessRequest ProcessStdio ProcessExit ProcessChild ProcessHost; do
  grep -q "$symbol" "$host_src" || fail "host contract missing $symbol"
done
for op in read_optional write remove take_stdin take_stdout take_stderr try_wait wait kill spawn; do
  grep -q "fn $op" "$host_src" || fail "host contract missing operation $op"
done
grep -q 'Sensitive<OsString>' "$host_src" || fail "process environment values are not sensitive by construction"
grep -q 'inherit_environment: false' "$host_src" || fail "ambient process environment must default off"
grep -q 'impl CodedError for FileAuthorityError' "$host_src" || fail "file authority coded error missing"
grep -q 'impl CodedError for ProcessContractError' "$host_src" || fail "process contract coded error missing"
! grep -q 'fn read(' "$host_src" || fail "mandatory file read is still unproven"
assert_no_match 'fn[[:space:]]+allows[[:space:]]*\(|fn[[:space:]]+(stdin|stdout|stderr)[[:space:]]*\(&mut self\)|fn[[:space:]]+run[[:space:]]*\(' \
  "host contract regained an unearned convenience API" "$host_src"
assert_no_match 'File(Store|Service|Manager)|HostServices|HostRegistry|ProcessManager|ProcessRegistry|NetworkHost|SecretHost|HostFuture' \
  "host contract regained an unproven abstraction" "$host_src"
echo "FILE_HOST_CONTRACT_OK"
echo "PROCESS_HOST_CONTRACT_OK"
echo "HOST_BOUNDARY_OK"

native_manifest="crates/audiacore-host-native/Cargo.toml"
native_src="crates/audiacore-host-native/src"
assert_dependencies "$native_manifest" "audiacore-host"
grep -Fq '[dev-dependencies]' "$native_manifest" || fail "native process proof test dependency missing"
grep -Fq 'audiacore-sensitive = { path = "../audiacore-sensitive" }' "$native_manifest" || fail "native sensitive dependency must remain test-only"
assert_no_match 'tokio|tracing|serde|reqwest|figment|audiacore-config|audiacore-core|audiacore-errors' \
  "native host contains upward/runtime/framework semantics" "$native_src"
grep -q '^mod file_store;' "$native_src/lib.rs" || fail "private atomic durability module missing"
! grep -Eq '^pub([[:space:]]*\([^)]*\))?[[:space:]]+mod[[:space:]]+file_store' "$native_src/lib.rs" \
  || fail "file_store must remain private"
assert_no_match 'pub struct File(Store|Service|Manager)|pub trait File(Store|Service|Manager)' \
  "native host recreated a public storage abstraction" "$native_src"
grep -q 'impl FileHost for NativeFileHost' "$native_src/lib.rs" || fail "NativeFileHost implementation missing"
grep -q 'fs::canonicalize' "$native_src/lib.rs" || fail "native file containment lacks canonicalization"
grep -q 'fs::symlink_metadata' "$native_src/lib.rs" || fail "native file leaf inspection missing"
grep -q 'OutsideAuthority' "$native_src/lib.rs" || fail "native file containment rejection missing"
grep -q 'SymbolicLinkWriteTarget' "$native_src/lib.rs" || fail "native file symlink-leaf rejection missing"
grep -q 'create_new(true)' "$native_src/file_store.rs" || fail "unique temporary creation missing"
grep -q 'sync_all()' "$native_src/file_store.rs" || fail "file durability sync missing"
grep -q 'fs::rename' "$native_src/file_store.rs" || fail "atomic replacement rename missing"
echo "NATIVE_FILE_HOST_OK"

process_src="$native_src/process.rs"
[[ -f "$process_src" ]] || fail "native process module missing"
grep -q '^mod process;' "$native_src/lib.rs" || fail "native process module not wired"
grep -q 'pub struct NativeProcessHost' "$process_src" || fail "NativeProcessHost missing"
grep -q 'pub enum NativeProcessError' "$process_src" || fail "effect-specific NativeProcessError missing"
grep -q 'impl ProcessHost for NativeProcessHost' "$process_src" || fail "ProcessHost native implementation missing"
grep -q 'impl ProcessChild for NativeProcess' "$process_src" || fail "ProcessChild native implementation missing"
grep -q 'fs::canonicalize' "$process_src" || fail "native process executable authorization lacks canonicalization"
grep -q 'ProgramNotAuthorized' "$process_src" || fail "native process authority rejection missing"
grep -q 'command.env_clear()' "$process_src" || fail "native process default environment clearing missing"
grep -q 'command.env(key, value.expose())' "$process_src" || fail "native process explicit environment insertion missing"
grep -q 'impl Drop for NativeProcess' "$process_src" || fail "owned direct-child cleanup missing"
grep -q 'self.child.kill()' "$process_src" || fail "direct-child kill missing"
grep -q 'self.child.wait()' "$process_src" || fail "direct-child reap missing"
! grep -q 'NativeHostError' "$process_src" || fail "process effects must keep an effect-specific error boundary"
assert_no_match 'ProcessManager|ProcessRegistry|tokio|HostFuture|descendant|process.?tree' \
  "native process regained an unproven manager/async/tree abstraction" "$process_src"
echo "NATIVE_PROCESS_HOST_OK"
echo "NATIVE_HOST_OK"

if [[ -d crates/audiacore-events ]]; then
  events_manifest="crates/audiacore-events/Cargo.toml"
  events_src="crates/audiacore-events/src/lib.rs"
  expected_event_deps="$(printf '%s\n' audiacore-core audiacore-errors)"
  assert_dependencies "$events_manifest" "$expected_event_deps"
  assert_no_dev_or_build_dependencies "$events_manifest"
  assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-host|audiacore-config|audiacore-host-native' \
    "events capability contains effects/runtime/config/native coupling" crates/audiacore-events
  for symbol in EventId EventStreamId CausationId EventSequence EventCursor EventPolicy EventEnvelope EventPage EventStream; do
    grep -q "$symbol" "$events_src" || fail "events capability missing $symbol"
  done
  grep -q 'pub fn bounded' "$events_src" || fail "EventPolicy bounded validation missing"
  grep -q 'pub fn page_after' "$events_src" || fail "typed incremental cursor paging missing"
  grep -q 'checked_add(1)' "$events_src" || fail "event sequence must reject exhaustion rather than wrap"
  grep -q 'impl CodedError for EventError' "$events_src" || fail "event capability failures lack stable coded identity"
  assert_no_match 'pub[[:space:]]+(struct|trait|enum)[[:space:]]+(EventBus|EventBroker|Publisher|Subscriber|Subscription|DurableEvent|Retry|Scheduler|Transport)' \
    "events capability defined an unearned delivery/broker/runtime abstraction" "$events_src"
  echo "EVENT_CAPABILITY_OK"
fi

if [[ -d crates/audiacore-workflow ]]; then
  workflow_manifest="crates/audiacore-workflow/Cargo.toml"
  workflow_src="crates/audiacore-workflow/src/lib.rs"
  assert_dependencies "$workflow_manifest" "audiacore-errors"
  assert_no_dev_or_build_dependencies "$workflow_manifest"
  assert_no_match 'std::(fs|env|process|net|time)|SystemTime|Instant|tokio|tracing|serde|reqwest|figment|audiacore-(core|events|config|host|host-native)' \
    "workflow capability contains effects/clock/runtime/upward coupling" crates/audiacore-workflow
  for symbol in WorkflowInstanceId WorkflowStatus WorkflowDefinition WorkflowTransition WorkflowInstance WorkflowReceipt WorkflowSnapshot WorkflowError; do
    grep -q "$symbol" "$workflow_src" || fail "workflow capability missing $symbol"
  done
  grep -q 'fn decide(' "$workflow_src" || fail "workflow deterministic decision boundary missing"
  grep -q 'effects: Vec<E>' "$workflow_src" || fail "workflow effects-as-data boundary missing"
  grep -q 'pub fn apply_at' "$workflow_src" || fail "workflow optimistic revision API missing"
  ! grep -q 'pub fn apply(' "$workflow_src" || fail "workflow regained redundant implicit-revision apply convenience"
  grep -q 'pub fn restore' "$workflow_src" || fail "workflow snapshot restoration boundary missing"
  grep -q 'checked_add(1)' "$workflow_src" || fail "workflow revision must reject exhaustion rather than wrap"
  ! grep -Eq 'self\.revision[[:space:]]*\+=' "$workflow_src" || fail "workflow revision must use checked monotonic increment"
  grep -q 'impl CodedError for WorkflowIdError' "$workflow_src" || fail "workflow identity validation lacks stable coded identity"
  grep -q 'impl<E> CodedError for WorkflowError<E>' "$workflow_src" || fail "workflow transition failures lack stable coded identity"
  assert_no_match 'pub[[:space:]]+(struct|trait|enum)[[:space:]]+(Workflow(Store|Repository|Persistence|Scheduler|Manager|Registry)|Retry|Backoff|Compensation|TaskExecutor)' \
    "workflow capability defined an unearned persistence/runtime/manager abstraction" "$workflow_src"
  echo "WORKFLOW_CAPABILITY_OK"
fi

echo "AUDIACORE_REVALIDATION_OK"
