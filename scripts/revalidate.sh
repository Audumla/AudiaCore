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
  docs/architecture/revalidation.md docs/architecture/layer-lock.md \
  .github/workflows/revalidation.yml; do
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
# Production libraries must never write unstructured diagnostics directly.
# Test targets are intentionally excluded because native child-process probes
# use stdout as the protocol surface being exercised.
cargo clippy --workspace --lib --locked -- \
  -D warnings \
  -D clippy::print_stdout \
  -D clippy::print_stderr \
  -D clippy::dbg_macro
cargo test --workspace --locked

echo "REPOSITORY_DISCIPLINE_OK"
echo "RUST_WORKSPACE_OK"

# Core: zero-dependency identity/composition kernel only.
core_manifest="crates/audiacore-core/Cargo.toml"
core_src="crates/audiacore-core/src"
[[ -f "$core_manifest" ]] || fail "core crate missing"
if grep -Eq 'dependencies\]$' "$core_manifest"; then
  fail "core must have zero normal/dev/build dependencies"
fi
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|config::|File|Process|Recipe|Provider|Policy' \
  "core contains later-layer/effect/runtime semantics" "$core_src"
for symbol in 'pub struct Application<C>' 'pub struct ExecutionContext' 'string_id!(ApplicationId' \
  'string_id!(ExecutionId' 'string_id!(CorrelationId'; do
  grep -Fq "$symbol" "$core_src/lib.rs" || fail "core missing $symbol"
done
echo "CORE_LAYER_OK"

# Stable error identity: no configured presentation or effects.
errors_manifest="crates/audiacore-errors/Cargo.toml"
errors_src="crates/audiacore-errors/src"
[[ -f "$errors_manifest" ]] || fail "stable error crate missing"
if grep -Eq 'dependencies\]$' "$errors_manifest"; then
  fail "stable error identity must have zero normal/dev/build dependencies"
fi
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-core|ErrorDefinition|canonical_message' \
  "stable error identity contains presentation/upward/effect semantics" "$errors_src"
for symbol in 'pub struct ErrorCode' 'pub enum ErrorCategory' 'pub trait CodedError'; do
  grep -Fq "$symbol" "$errors_src/lib.rs" || fail "stable error identity missing $symbol"
done
[[ ! -e crates/audiacore-errors/errors.yaml ]] || fail "stable identity crate must not own configured messages"

# Every production code is owned exactly once by its crate-local configured catalogue.
python - <<'PY'
from pathlib import Path
import re
import sys

code_re = re.compile(
    r'const\s+[A-Z0-9_]+\s*:\s*ErrorCode\s*=\s*ErrorCode::new\("([A-Z][A-Z0-9-]*-\d{3})"\);'
)
yaml_code_re = re.compile(r'^([A-Z]{2,}(?:-[A-Z][A-Z0-9]*)+-\d{3}):\s*$', re.MULTILINE)

all_yaml = {}
problems = []
for crate in sorted(Path("crates").glob("audiacore-*")):
    rust_codes = set()
    src = crate / "src"
    if src.is_dir():
        for path in src.rglob("*.rs"):
            rust_codes.update(code_re.findall(path.read_text()))

    yaml_codes = set()
    catalogue = crate / "errors.yaml"
    if catalogue.is_file():
        matches = yaml_code_re.findall(catalogue.read_text())
        if len(matches) != len(set(matches)):
            problems.append(f"{catalogue}: duplicate top-level stable code")
        yaml_codes = set(matches)
        for code in yaml_codes:
            if code in all_yaml:
                problems.append(f"{code}: defined by both {all_yaml[code]} and {catalogue}")
            all_yaml[code] = catalogue

    if rust_codes != yaml_codes:
        missing = sorted(rust_codes - yaml_codes)
        extra = sorted(yaml_codes - rust_codes)
        if missing:
            problems.append(f"{crate}: production codes missing from errors.yaml: {missing}")
        if extra:
            problems.append(f"{crate}: errors.yaml codes without production identity: {extra}")

if problems:
    print("REVALIDATION_FAIL: error catalogue ownership/coverage mismatch", file=sys.stderr)
    for problem in problems:
        print(f"  - {problem}", file=sys.stderr)
    raise SystemExit(1)
PY

echo "ERROR_IDENTITY_OK"

# Sensitive values remain a tiny coded pure primitive.
sensitive_dir="crates/audiacore-sensitive"
assert_dependencies "$sensitive_dir/Cargo.toml" "audiacore-errors"
assert_no_dev_or_build_dependencies "$sensitive_dir/Cargo.toml"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-(core|host|config)' \
  "sensitive values contain upward/effect/runtime semantics" "$sensitive_dir/src"
grep -q 'pub struct Sensitive<T>' "$sensitive_dir/src/lib.rs" || fail "Sensitive<T> missing"
grep -q 'impl CodedError for' "$sensitive_dir/src/lib.rs" || fail "sensitive failures need stable identity"

# Reconciliation is only desired/observed -> effect intent. No ownership/resource/error vocabulary.
reconcile_manifest="crates/audiacore-reconcile/Cargo.toml"
reconcile_src="crates/audiacore-reconcile/src/lib.rs"
[[ -f "$reconcile_src" ]] || fail "reconcile source missing"
assert_no_dev_or_build_dependencies "$reconcile_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-(core|host|config)|ErrorCode|CodedError|ResourceId|OwnerId|Ownership|Authority|Policy' \
  "reconcile acquired resource/ownership/effect/policy semantics" "$reconcile_src"
[[ ! -e crates/audiacore-reconcile/errors.yaml ]] || fail "pure reconcile must not own an error catalogue"
grep -q 'pub enum ReconcileAction<T>' "$reconcile_src" || fail "reconciliation effect-as-data missing"
grep -q 'pub fn plan<T>' "$reconcile_src" || fail "generic reconciliation planner missing"
echo "PURE_RECONCILIATION_OK"

# Templates are mapping-only presentation mechanics.
template_manifest="crates/audiacore-template/Cargo.toml"
template_src="crates/audiacore-template/src/lib.rs"
assert_dependencies "$template_manifest" "$(printf '%s\n' audiacore-errors serde_json)"
assert_no_dev_or_build_dependencies "$template_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-(core|host|config)' \
  "template contains effects/upward semantics" crates/audiacore-template/src
for symbol in 'pub struct Template' 'resolve_path' 'TemplateContext'; do
  grep -q "$symbol" "$template_src" || fail "template missing $symbol"
done
! grep -Fq 'find("{{")' "$template_src" || fail "legacy flat double-brace template syntax returned"
echo "PURE_FOUNDATION_PRIMITIVES_OK"

# Configured error presentation is caller-owned and effect-free.
catalog_manifest="crates/audiacore-error-catalog/Cargo.toml"
catalog_src="crates/audiacore-error-catalog/src/lib.rs"
assert_dependencies "$catalog_manifest" "$(printf '%s\n' audiacore-errors audiacore-template serde yaml_serde)"
assert_no_dev_or_build_dependencies "$catalog_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-(core|config|host|host-native|events|workflow|time|managed-config)' \
  "error catalogue contains discovery/effects/upward semantics" "$catalog_src"
for symbol in ErrorDefinition ErrorCatalogue RenderedError ErrorCatalogueError; do
  grep -q "$symbol" "$catalog_src" || fail "configured error catalogue missing $symbol"
done
for op in register_yaml overlay_yaml render; do
  grep -q "pub fn $op" "$catalog_src" || fail "error catalogue missing $op"
done
assert_no_match 'OnceLock|OnceCell|lazy_static|Global.*Catalogue|ErrorRegistry' \
  "error catalogue regained global registry semantics" "$catalog_src"
echo "CONFIGURED_ERROR_PRESENTATION_OK"

# Configuration resolves already-acquired content; it does not acquire sources or own policy.
config_manifest="crates/audiacore-config/Cargo.toml"
config_src="crates/audiacore-config/src/lib.rs"
assert_dependencies "$config_manifest" "$(printf '%s\n' audiacore-errors serde toml)"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|figment|audiacore-(core|host)|Policy|from_env|from_file|read_to_string' \
  "config acquired source effects/provider/policy/upward semantics" crates/audiacore-config/src
for symbol in ConfigLayerId ConfigRevision 'ResolvedConfig<T>' ConfigLayers; do
  grep -q "$symbol" "$config_src" || fail "configuration resolver missing $symbol"
done
grep -q 'pub fn merge_toml' "$config_src" || fail "in-memory TOML merge missing"
! grep -q 'into_value' "$config_src" || fail "config must not silently discard provenance"
echo "CONFIG_RESOLUTION_OK"

# Host ports describe effects and explicit authorization, never app policy or native mechanics.
host_manifest="crates/audiacore-host/Cargo.toml"
host_src="crates/audiacore-host/src/lib.rs"
assert_dependencies "$host_manifest" "$(printf '%s\n' audiacore-errors audiacore-sensitive)"
assert_no_dev_or_build_dependencies "$host_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-(config|core)|Recipe|ManagedContent|PackageManager' \
  "host contract contains native/config/application semantics" crates/audiacore-host/src
for symbol in FileReadAuthority FileWriteAuthority FileHost ProcessAuthority ProcessRequest ProcessStdio ProcessExit ProcessChild ProcessHost; do
  grep -q "$symbol" "$host_src" || fail "host contract missing $symbol"
done
grep -q 'Sensitive<OsString>' "$host_src" || fail "process environment values must be sensitive"
grep -q 'inherit_environment: false' "$host_src" || fail "ambient process environment must default off"
assert_no_match 'ServiceLocator|HostServices|HostRegistry|FileManager|ProcessManager|ProcessRegistry' \
  "host contract regained container/manager semantics" "$host_src"
echo "HOST_BOUNDARY_OK"

# Native host contains OS mechanics only. Filesystem effects are relative to a
# cap-std directory capability; callers and public host contracts do not see
# cap-std types.
native_manifest="crates/audiacore-host-native/Cargo.toml"
native_src="crates/audiacore-host-native/src"
assert_dependencies "$native_manifest" "$(printf '%s\n' audiacore-host cap-std)"
assert_no_match 'tokio|tracing|serde|reqwest|figment|audiacore-(config|core|errors)|Recipe|Policy|Provider' \
  "native host contains upward/runtime/application semantics" "$native_src"
grep -q 'impl FileHost for NativeFileHost' "$native_src/lib.rs" || fail "NativeFileHost implementation missing"
grep -q 'Dir::open_ambient_dir' "$native_src/lib.rs" || fail "native file authority must acquire a directory capability"
grep -q 'dir.symlink_metadata' "$native_src/lib.rs" || fail "native file leaf inspection must be capability-relative"
grep -q 'dir.read' "$native_src/lib.rs" || fail "native file reads must be capability-relative"
grep -q 'dir.remove_file' "$native_src/lib.rs" || fail "native file removals must be capability-relative"
grep -q 'dir.open_with' "$native_src/file_store.rs" || fail "temporary file creation must be capability-relative"
grep -q 'dir.rename' "$native_src/file_store.rs" || fail "atomic replacement rename must be capability-relative"
grep -q 'create_new(true)' "$native_src/file_store.rs" || fail "unique temporary creation missing"
grep -q 'sync_all()' "$native_src/file_store.rs" || fail "file durability sync missing"
for file in "$native_src/lib.rs" "$native_src/file_store.rs"; do
  if sed '/^#\[cfg(test)\]/,$d' "$file" | grep -Eq 'fs::(canonicalize|read|write|remove_file|symlink_metadata|rename)'; then
    fail "$file reintroduced ambient std::fs target operations after capability acquisition"
  fi
done
process_src="$native_src/process.rs"
grep -q 'impl ProcessHost for NativeProcessHost' "$process_src" || fail "native ProcessHost implementation missing"
grep -q 'impl ProcessChild for NativeProcess' "$process_src" || fail "native ProcessChild implementation missing"
grep -q 'command.env_clear()' "$process_src" || fail "native process must clear ambient environment by default"
grep -q 'impl Drop for NativeProcess' "$process_src" || fail "owned direct-child cleanup missing"
assert_no_match 'ProcessManager|ProcessRegistry|HostFuture|descendant|process.?tree' \
  "native process regained manager/async/tree semantics" "$process_src"
echo "NATIVE_HOST_OK"

# Reusable capabilities remain independent and effect-free unless explicitly using host ports.
events_manifest="crates/audiacore-events/Cargo.toml"
events_src="crates/audiacore-events/src/lib.rs"
assert_dependencies "$events_manifest" "$(printf '%s\n' audiacore-core audiacore-errors)"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-(host|config|host-native)' \
  "events contains effects/runtime/config coupling" crates/audiacore-events/src
for symbol in EventId EventStreamId CausationId EventSequence EventCursor EventPolicy EventEnvelope EventPage EventStream; do
  grep -q "$symbol" "$events_src" || fail "events missing $symbol"
done
assert_no_match 'EventBus|EventBroker|Publisher|Subscriber|Subscription|Retry|Scheduler|Transport' \
  "events regained broker/runtime semantics" "$events_src"
echo "EVENT_CAPABILITY_OK"

workflow_manifest="crates/audiacore-workflow/Cargo.toml"
workflow_src="crates/audiacore-workflow/src/lib.rs"
assert_dependencies "$workflow_manifest" "audiacore-errors"
assert_no_match 'std::(fs|env|process|net|time)|SystemTime|Instant|tokio|tracing|serde|reqwest|figment|audiacore-(core|events|config|host|host-native)' \
  "workflow contains effects/clock/runtime/upward coupling" crates/audiacore-workflow/src
for symbol in WorkflowInstanceId WorkflowStatus WorkflowDefinition WorkflowTransition WorkflowInstance WorkflowReceipt WorkflowSnapshot WorkflowError; do
  grep -q "$symbol" "$workflow_src" || fail "workflow missing $symbol"
done
assert_no_match 'Workflow(Store|Repository|Persistence|Scheduler|Manager|Registry)|Retry|Backoff|Compensation|TaskExecutor' \
  "workflow regained persistence/runtime/manager semantics" "$workflow_src"
echo "WORKFLOW_CAPABILITY_OK"

time_manifest="crates/audiacore-time/Cargo.toml"
time_src="crates/audiacore-time/src/lib.rs"
assert_dependencies "$time_manifest" "audiacore-errors"
assert_no_match 'std::(fs|env|process|net|time)|SystemTime|Instant|tokio|tracing|serde|reqwest|figment|audiacore-(core|events|workflow|config|host|host-native)' \
  "time contains effects/clock/runtime/upward coupling" crates/audiacore-time/src
for symbol in Timestamp Deadline TimerId TimerSet; do
  grep -q "$symbol" "$time_src" || fail "time missing $symbol"
done
assert_no_match 'Clock|TimeProvider|TimerManager|TimerRegistry|Scheduler|Task|Sleep|Retry|Backoff' \
  "time regained clock/scheduler/runtime semantics" "$time_src"
echo "TIME_CAPABILITY_OK"

# Current managed-config is deliberately only a whole-file capability.
managed_manifest="crates/audiacore-managed-config/Cargo.toml"
managed_src="crates/audiacore-managed-config/src/lib.rs"
assert_dependencies "$managed_manifest" "$(printf '%s\n' audiacore-errors audiacore-host audiacore-reconcile)"
assert_no_dev_or_build_dependencies "$managed_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|figment|audiacore-(core|events|workflow|time|config|host-native)|OwnerId|OwnershipMismatch|ManagedContent|Policy' \
  "managed whole-file capability contains source/native/ownership/upward semantics" crates/audiacore-managed-config/src
for symbol in ManagedConfigTarget ManagedConfigPlan ManagedConfigApplyResult ManagedConfigError; do
  grep -q "$symbol" "$managed_src" || fail "managed whole-file capability missing $symbol"
done
grep -q 'target: ManagedConfigTarget' "$managed_src" || fail "whole-file plan must bind the exact target"
grep -q 'host.read_optional' "$managed_src" || fail "whole-file observation must use FileHost"
grep -q 'reconcile_presence' "$managed_src" || fail "whole-file planning must delegate to pure reconcile"
grep -q 'host.write(authority, plan.target().path()' "$managed_src" || fail "apply must use the plan-bound target"
grep -q 'host.remove(authority, plan.target().path()' "$managed_src" || fail "delete must use the plan-bound target"
assert_no_match 'Parser|Watcher|Scheduler|Retry|Backoff|Cas|CAS|Manager|Registry|Receipt' \
  "managed whole-file capability regained unearned higher-level semantics" "$managed_src"
echo "MANAGED_WHOLE_FILE_CAPABILITY_OK"

# Global semantic lock: production code must not bypass the tracing/host/config boundaries.
assert_no_match 'set_global_default|tracing_subscriber::.*\.init\(|tracing_subscriber::.*try_init\(' \
  "library code owns a global tracing subscriber" crates/*/src
assert_no_match 'ServiceRegistry|ProviderRegistry|PolicyRegistry|ServiceLocator|DependencyContainer|GlobalRuntime|GlobalContext' \
  "registry/container semantics returned" crates/*/src

grep -q 'Sources provide data' docs/architecture/layer-lock.md || fail "layer-lock governing rule missing"
grep -q 'Dependency health and build-versus-buy' docs/architecture/layer-lock.md || fail "dependency-health contract missing"
grep -q 'Figment was reconsidered' docs/architecture/layer-lock.md || fail "Figment maintenance decision not recorded"
grep -q 'rust-cli/config-rs' docs/architecture/layer-lock.md || fail "live config library alternative not recorded"

echo "SEMANTIC_LAYER_LOCK_OK"
echo "AUDIACORE_REVALIDATION_OK"
