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
  docs/architecture/target-state.md docs/architecture/roadmap.md \
  docs/architecture/dependencies.md .github/workflows/revalidation.yml; do
  [[ -f "$path" ]] || fail "missing required repository file: $path"
done

for path in CLAUDE.md COPILOT.md GEMINI.md QWEN.md pyproject.toml uv.lock \
  package.json package-lock.json Makefile .audiagentic .agents .qwen archive src tests \
  crates/audiacore-application scripts/revalidate-stage7.sh; do
  [[ ! -e "$path" ]] || fail "legacy/proof-only repository surface present: $path"
done

if find . -path './.git' -prune -o -path './target' -prune -o \
  \( -name '*.pyc' -o -name '*.rs.bk' -o -name '.DS_Store' \) -print -quit | grep -q .; then
  fail "generated/local artifact present in validation tree"
fi

assert_no_match 'audiacore-managed-config|ManagedConfig|managed_config|IO-MCONFIG|revalidate-stage7' \
  "retired terminology/proof scaffolding returned" \
  Cargo.toml README.md AGENTS.md crates .github \
  docs/architecture/layer-lock.md docs/architecture/target-state.md \
  docs/architecture/roadmap.md docs/architecture/dependencies.md

assert_no_match 'tracing(-subscriber)?[[:space:]]*=' \
  "tracing dependencies returned without an active application/executable consumer" Cargo.toml crates

rustc --version
cargo --version
cargo fmt --all -- --check
cargo clippy --workspace --all-targets --locked -- -D warnings
cargo clippy --workspace --lib --locked -- \
  -D warnings -D clippy::print_stdout -D clippy::print_stderr -D clippy::dbg_macro
cargo test --workspace --locked

echo "RUST_WORKSPACE_OK"

# Core: identity and opaque composition only.
core_manifest="crates/audiacore-core/Cargo.toml"
core_src="crates/audiacore-core/src"
if grep -Eq 'dependencies\]$' "$core_manifest"; then
  fail "core must have zero normal/dev/build dependencies"
fi
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|FileHost|ProcessHost|Provider|Policy|Plugin|Registry' \
  "core contains later-layer/effect/runtime semantics" "$core_src"
for symbol in 'pub struct Application<C>' 'pub struct ExecutionContext' \
  'string_id!(ApplicationId' 'string_id!(ExecutionId' 'string_id!(CorrelationId'; do
  grep -Fq "$symbol" "$core_src/lib.rs" || fail "core missing $symbol"
done

# Stable errors: identity/category only.
errors_manifest="crates/audiacore-errors/Cargo.toml"
errors_src="crates/audiacore-errors/src"
if grep -Eq 'dependencies\]$' "$errors_manifest"; then
  fail "stable error identity must have zero normal/dev/build dependencies"
fi
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|ErrorDefinition|canonical_message' \
  "stable errors contain presentation/effect semantics" "$errors_src"
for symbol in 'pub struct ErrorCode' 'pub enum ErrorCategory' 'pub trait CodedError'; do
  grep -Fq "$symbol" "$errors_src/lib.rs" || fail "stable errors missing $symbol"
done
[[ ! -e crates/audiacore-errors/errors.yaml ]] || fail "stable error identity must not own configured messages"

# Production coded errors and owner-local catalogues must match exactly.
python - <<'PY'
from pathlib import Path
import re
import sys

code_re = re.compile(r'const\s+[A-Z0-9_]+\s*:\s*ErrorCode\s*=\s*ErrorCode::new\("([A-Z][A-Z0-9-]*-\d{3})"\);')
yaml_code_re = re.compile(r'^([A-Z]{2,}(?:-[A-Z][A-Z0-9]*)+-\d{3}):\s*$', re.MULTILINE)
problems = []
all_yaml = {}
for crate in sorted(Path("crates").glob("audiacore-*")):
    rust_codes = set()
    src = crate / "src"
    if src.is_dir():
        for path in src.rglob("*.rs"):
            rust_codes.update(code_re.findall(path.read_text()))

    yaml_codes = set()
    catalogue = crate / "errors.yaml"
    if catalogue.is_file():
        text = catalogue.read_text()
        if re.search(r'^\s+kind:', text, re.MULTILINE):
            problems.append(f"{catalogue}: category must not be duplicated as kind")
        matches = yaml_code_re.findall(text)
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
    print("REVALIDATION_FAIL: error catalogue ownership/schema mismatch", file=sys.stderr)
    for problem in problems:
        print(f"  - {problem}", file=sys.stderr)
    raise SystemExit(1)
PY

echo "ERROR_CONTRACT_OK"

# Pure foundation primitives.
sensitive_dir="crates/audiacore-sensitive"
assert_dependencies "$sensitive_dir/Cargo.toml" "audiacore-errors"
assert_no_dev_or_build_dependencies "$sensitive_dir/Cargo.toml"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|audiacore-(core|host|config)' \
  "sensitive values contain upward/effect/runtime semantics" "$sensitive_dir/src"
for symbol in 'pub struct Sensitive<T>' 'pub struct SensitiveKey' 'pub fn redact_text'; do
  grep -Fq "$symbol" "$sensitive_dir/src/lib.rs" || fail "sensitive values missing $symbol"
done

reconcile_manifest="crates/audiacore-reconcile/Cargo.toml"
reconcile_src="crates/audiacore-reconcile/src/lib.rs"
assert_no_dev_or_build_dependencies "$reconcile_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|audiacore-|ErrorCode|CodedError|ResourceId|OwnerId|Ownership|Authority|Policy' \
  "reconcile acquired resource/ownership/effect/policy semantics" "$reconcile_src"
grep -Fq 'pub enum ReconcileAction<T>' "$reconcile_src" || fail "reconciliation action missing"
grep -Fq 'pub fn plan<T>' "$reconcile_src" || fail "reconciliation planner missing"

template_manifest="crates/audiacore-template/Cargo.toml"
template_src="crates/audiacore-template/src/lib.rs"
assert_dependencies "$template_manifest" "$(printf '%s\n' audiacore-errors serde_json)"
assert_no_dev_or_build_dependencies "$template_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|audiacore-(core|host|config)' \
  "template contains effects/upward semantics" crates/audiacore-template/src
for symbol in 'pub struct Template' 'TemplateContext' 'pub fn render'; do
  grep -Fq "$symbol" "$template_src" || fail "template missing $symbol"
done

catalog_manifest="crates/audiacore-error-catalog/Cargo.toml"
catalog_src="crates/audiacore-error-catalog/src/lib.rs"
assert_dependencies "$catalog_manifest" "$(printf '%s\n' audiacore-errors audiacore-template serde yaml_serde)"
assert_no_dev_or_build_dependencies "$catalog_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|audiacore-(core|config|host|host-native|events|workflow|time|managed-content)' \
  "error catalogue contains discovery/effects/upward semantics" "$catalog_src"
for symbol in ErrorDefinition ErrorCatalogue RenderedError ErrorCatalogueError; do
  grep -Fq "$symbol" "$catalog_src" || fail "error catalogue missing $symbol"
done
for op in register_yaml overlay_yaml render; do
  grep -Fq "pub fn $op" "$catalog_src" || fail "error catalogue missing $op"
done
assert_no_match 'kind:[[:space:]]*String|raw.kind|InvalidKind|OnceLock|OnceCell|lazy_static|ErrorRegistry' \
  "error catalogue regained duplicate category/global registry semantics" "$catalog_src"

echo "FOUNDATION_PRIMITIVES_OK"

# Configuration resolves content already acquired by the caller.
config_manifest="crates/audiacore-config/Cargo.toml"
config_src="crates/audiacore-config/src/lib.rs"
assert_dependencies "$config_manifest" "$(printf '%s\n' audiacore-errors serde toml)"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|reqwest|audiacore-(core|host)|Policy|from_env|from_file|read_to_string' \
  "config acquired source effects/policy/upward semantics" crates/audiacore-config/src
for symbol in ConfigLayerId ConfigRevision 'ResolvedConfig<T>' ConfigLayers; do
  grep -Fq "$symbol" "$config_src" || fail "config missing $symbol"
done
grep -Fq 'pub fn merge_toml' "$config_src" || fail "in-memory TOML merge missing"

# Host contracts and native adapters.
host_manifest="crates/audiacore-host/Cargo.toml"
host_src="crates/audiacore-host/src/lib.rs"
assert_dependencies "$host_manifest" "$(printf '%s\n' audiacore-errors audiacore-sensitive)"
assert_no_dev_or_build_dependencies "$host_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|audiacore-(config|core)|ManagedContent|Recipe|Provider|PackageManager' \
  "host contract contains native/application/content semantics" crates/audiacore-host/src
for symbol in FileReadAuthority FileWriteAuthority FileHost ProcessAuthority ProcessRequest ProcessStdio ProcessExit ProcessChild ProcessHost; do
  grep -Fq "$symbol" "$host_src" || fail "host contract missing $symbol"
done
grep -Fq 'Sensitive<OsString>' "$host_src" || fail "process environment values must be sensitive"
grep -Fq 'inherit_environment: false' "$host_src" || fail "ambient process environment must default off"

native_manifest="crates/audiacore-host-native/Cargo.toml"
native_src="crates/audiacore-host-native/src"
assert_dependencies "$native_manifest" "$(printf '%s\n' audiacore-host cap-std)"
assert_no_match 'tokio|tracing|serde|reqwest|audiacore-(config|core|errors)|ManagedContent|Recipe|Policy|Provider' \
  "native host contains upward/application/content semantics" "$native_src"
grep -Fq 'impl FileHost for NativeFileHost' "$native_src/lib.rs" || fail "NativeFileHost missing"
grep -Fq 'Dir::open_ambient_dir' "$native_src/lib.rs" || fail "native file authority must acquire Dir capability"
grep -Fq 'dir.read' "$native_src/lib.rs" || fail "native file reads must be capability-relative"
grep -Fq 'dir.remove_file' "$native_src/lib.rs" || fail "native file removals must be capability-relative"
grep -Fq 'dir.open_with' "$native_src/file_store.rs" || fail "temporary creation must be capability-relative"
grep -Fq 'dir.rename' "$native_src/file_store.rs" || fail "atomic rename must be capability-relative"
for file in "$native_src/lib.rs" "$native_src/file_store.rs"; do
  if sed '/^#\[cfg(test)\]/,$d' "$file" | grep -Eq 'fs::(canonicalize|read|write|remove_file|symlink_metadata|rename)'; then
    fail "$file reintroduced ambient std::fs target operations"
  fi
done
grep -Fq 'impl ProcessHost for NativeProcessHost' "$native_src/process.rs" || fail "NativeProcessHost missing"
grep -Fq 'command.env_clear()' "$native_src/process.rs" || fail "native process must clear ambient environment by default"
grep -Fq 'impl Drop for NativeProcess' "$native_src/process.rs" || fail "owned child cleanup missing"

echo "HOST_BOUNDARIES_OK"

# Reusable capabilities.
events_manifest="crates/audiacore-events/Cargo.toml"
events_src="crates/audiacore-events/src/lib.rs"
assert_dependencies "$events_manifest" "$(printf '%s\n' audiacore-core audiacore-errors)"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|audiacore-(host|config|host-native)' \
  "events contains effects/runtime/config coupling" crates/audiacore-events/src
for symbol in EventId EventStreamId CausationId EventSequence EventCursor EventPolicy EventEnvelope EventPage EventStream; do
  grep -Fq "$symbol" "$events_src" || fail "events missing $symbol"
done
assert_no_match 'EventBus|EventBroker|Publisher|Subscriber|Subscription|Retry|Scheduler|Transport' \
  "events regained broker/runtime semantics" "$events_src"

workflow_manifest="crates/audiacore-workflow/Cargo.toml"
workflow_src="crates/audiacore-workflow/src/lib.rs"
assert_dependencies "$workflow_manifest" "audiacore-errors"
assert_no_match 'std::(fs|env|process|net|time)|SystemTime|Instant|tokio|tracing|serde|reqwest|audiacore-(core|events|config|host|host-native)' \
  "workflow contains effects/clock/runtime/upward coupling" crates/audiacore-workflow/src
for symbol in WorkflowInstanceId WorkflowStatus WorkflowDefinition WorkflowTransition WorkflowInstance WorkflowReceipt WorkflowSnapshot WorkflowError; do
  grep -Fq "$symbol" "$workflow_src" || fail "workflow missing $symbol"
done
assert_no_match 'Workflow(Store|Repository|Persistence|Scheduler|Manager|Registry)|Retry|Backoff|Compensation|TaskExecutor' \
  "workflow regained persistence/runtime/manager semantics" "$workflow_src"

time_manifest="crates/audiacore-time/Cargo.toml"
time_src="crates/audiacore-time/src/lib.rs"
assert_dependencies "$time_manifest" "audiacore-errors"
assert_no_match 'std::(fs|env|process|net|time)|SystemTime|Instant|tokio|tracing|serde|reqwest|audiacore-(core|events|workflow|config|host|host-native)' \
  "time contains effects/clock/runtime/upward coupling" crates/audiacore-time/src
for symbol in Timestamp Deadline TimerId TimerSet; do
  grep -Fq "$symbol" "$time_src" || fail "time missing $symbol"
done
assert_no_match 'Clock|TimeProvider|TimerManager|TimerRegistry|Scheduler|Task|Sleep|Retry|Backoff' \
  "time regained clock/scheduler/runtime semantics" "$time_src"

managed_manifest="crates/audiacore-managed-content/Cargo.toml"
managed_src="crates/audiacore-managed-content/src/lib.rs"
assert_dependencies "$managed_manifest" "$(printf '%s\n' audiacore-errors audiacore-host audiacore-reconcile)"
assert_no_dev_or_build_dependencies "$managed_manifest"
assert_no_match 'std::(fs|env|process|net)|tokio|tracing|serde|reqwest|audiacore-(core|events|workflow|time|config|host-native)|OwnerId|OwnershipMismatch|ContributionId|Policy' \
  "Managed Content whole-file slice contains source/native/ownership/upward semantics" crates/audiacore-managed-content/src
for symbol in ManagedContentTarget ManagedContentPlan ManagedContentApplyResult ManagedContentError; do
  grep -Fq "$symbol" "$managed_src" || fail "Managed Content slice missing $symbol"
done
grep -Fq 'reconcile_presence' "$managed_src" || fail "Managed Content planning must delegate to reconcile"
grep -Fq 'host.write(authority, plan.target().path()' "$managed_src" || fail "Managed Content apply must use plan-bound target"
grep -Fq 'host.remove(authority, plan.target().path()' "$managed_src" || fail "Managed Content delete must use plan-bound target"
assert_no_match 'Parser|Watcher|Scheduler|Retry|Backoff|Cas|CAS|Manager|Registry|Receipt' \
  "Managed Content whole-file slice regained unearned semantics" "$managed_src"

echo "CAPABILITY_LAYERS_OK"

# Global locks.
assert_no_match 'set_global_default|tracing_subscriber::.*\.init\(|tracing_subscriber::.*try_init\(' \
  "library code owns a global tracing subscriber" crates/*/src
assert_no_match 'ServiceRegistry|ProviderRegistry|PolicyRegistry|ServiceLocator|DependencyContainer|GlobalRuntime|GlobalContext' \
  "registry/container semantics returned" crates/*/src

grep -Fq 'Sources provide data' docs/architecture/layer-lock.md || fail "governing layer rule missing"
grep -Fq 'no current application-assembly crate' docs/architecture/layer-lock.md || fail "application assembly boundary missing"
grep -Fq 'Application assembly' docs/architecture/target-state.md || fail "target application assembly missing"
grep -Fq 'External component sourcing' docs/architecture/target-state.md || fail "target external sourcing missing"
grep -Fq 'Managed package lifecycle' docs/architecture/target-state.md || fail "target package lifecycle missing"
grep -Fq 'Managed Content partial/structured ownership' docs/architecture/target-state.md || fail "target Managed Content expansion missing"

echo "AUDIACORE_REVALIDATION_OK"
