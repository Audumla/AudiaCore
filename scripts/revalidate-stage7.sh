#!/usr/bin/env bash
set -euo pipefail

repo_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
cd "$repo_root"

fail() {
  echo "STAGE7_REVALIDATION_FAIL: $*" >&2
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

dev_dependencies() {
  awk '
    /^\[dev-dependencies\]$/ { in_deps=1; next }
    /^\[/ { in_deps=0 }
    in_deps && /^[A-Za-z0-9_-]+[[:space:]]*=/ {
      key=$1; sub(/[[:space:]]*=.*$/, "", key); print key
    }
  ' "$1"
}

app_dir="crates/audiacore-application"
manifest="$app_dir/Cargo.toml"
src="$app_dir/src/lib.rs"
proof="$app_dir/tests/stage7_proof.rs"

[[ -f "$manifest" ]] || fail "application composition crate missing"
[[ -f "$src" ]] || fail "application composition source missing"
[[ -f "$proof" ]] || fail "Stage 7 end-to-end proof missing"

expected_normal="$(printf '%s\n' \
  audiacore-core \
  audiacore-error-catalog \
  audiacore-errors \
  audiacore-host \
  audiacore-managed-content \
  audiacore-sensitive \
  audiacore-template \
  tracing)"
actual_normal="$(normal_dependencies "$manifest")"
[[ "$actual_normal" == "$expected_normal" ]] \
  || fail "application normal dependencies changed; expected [$expected_normal], found [$actual_normal]"

expected_dev="$(printf '%s\n' \
  audiacore-config \
  audiacore-host-native \
  serde \
  tracing-subscriber)"
actual_dev="$(dev_dependencies "$manifest")"
[[ "$actual_dev" == "$expected_dev" ]] \
  || fail "application proof dependencies changed; expected [$expected_dev], found [$actual_dev]"

for symbol in ManagedContentRequest ManagedContentComposition MessageContext PresentedError; do
  grep -q "pub struct $symbol" "$src" || fail "Stage 7 application edge missing $symbol"
done
! grep -q 'ManagedContentPolicy' "$src" || fail "capability request is mislabeled as application policy"
grep -q 'pub fn execute_managed_content' "$src" || fail "Stage 7 execution proof missing"
grep -q 'pub fn present_error' "$src" || fail "configured error presentation edge missing"
grep -q 'pub fn redacted_value' "$src" || fail "sensitive message projection seam missing"
grep -q 'tracing::info_span!' "$src" || fail "structured execution span missing"
grep -q 'error_code = error.code().as_str()' "$src" || fail "coded application failure tracing missing"
grep -q 'application_id = %application.id()' "$src" || fail "application identity not carried into tracing"
grep -q 'execution_id = %execution.execution_id()' "$src" || fail "execution identity not carried into tracing"
grep -q 'correlation_id = %execution.correlation_id()' "$src" || fail "correlation identity not carried into tracing"
grep -q 'outcome = "success"' "$src" || fail "successful application outcome is not structured"
grep -q 'outcome = "failure"' "$src" || fail "failed application outcome is not structured"
grep -q 'outcome = "degraded"' "$src" || fail "degraded presentation outcome is not structured"
grep -q 'result = managed_content_result(result)' "$src" || fail "Managed Content result is not emitted as a stable audit value"
! grep -Eq 'result[[:space:]]*=[[:space:]]*\?' "$src" || fail "audit result must not use Debug formatting"
! grep -Eq 'presentation_error[[:space:]]*=[[:space:]]*%' "$src" || fail "presentation fallback must not emit arbitrary diagnostic text at WARN"

if grep -Eq 'ServiceRegistry|ProviderRegistry|PolicyRegistry|HostServices|ServiceLocator|DependencyContainer|Global(Context|Runtime|Registry)|OnceLock|OnceCell|lazy_static|set_global_default|\.init\(\)|try_init\(' "$src"; then
  fail "Stage 7 introduced global/container/registry infrastructure"
fi
if grep -Eq 'tokio|async[[:space:]]+fn|Scheduler|Manager' "$src"; then
  fail "Stage 7 introduced an unearned runtime/manager abstraction"
fi

# Capability requests must remain source-independent: configuration and the
# concrete native host are proof-only dependencies, not part of reusable
# request/composition APIs.
if [[ "$actual_normal" == *"audiacore-config"* || "$actual_normal" == *"audiacore-host-native"* ]]; then
  fail "request/composition became coupled to configuration source or native implementation"
fi
grep -q 'ConfigLayers' "$proof" || fail "resolved configuration is not exercised at the application edge"
grep -q 'NativeFileHost' "$proof" || fail "real native effect is not exercised by Stage 7 proof"
grep -q 'FileReadAuthority' "$proof" || fail "read authority is not explicitly supplied"
grep -q 'FileWriteAuthority' "$proof" || fail "write authority is not explicitly supplied"
grep -q 'tracing::subscriber::with_default' "$proof" || fail "observability subscriber is not edge-owned"
grep -q 'fs::read' "$proof" || fail "end-to-end proof does not verify the real native file effect"
grep -q 'IO-MCONTENT-001' "$proof" || fail "real native failure does not assert stable Managed Content error identity"
grep -q 'present_error(' "$proof" || fail "real coded failure is not passed through configured presentation"
grep -Fq 'application.composition().errors()' "$proof" || fail "configured presentation is not using the caller-owned application catalogue"
grep -q 'request_from_config' "$proof" || fail "resolved settings are not converted into a capability request"
grep -q 'direct_request' "$proof" || fail "source-independent direct capability request is not proven"
grep -q 'result=.*created' "$proof" || fail "proof does not assert stable lowercase result logging"
grep -q 'outcome=.*success' "$proof" || fail "proof does not assert structured success outcome"

grep -q 'sensitive_message_values_are_redacted_without_exposure' "$src" \
  || fail "sensitive error-message projection is untested"
grep -q 'presentation_failure_preserves_original_code_without_diagnostic_text' "$src" \
  || fail "configured-presentation fallback identity is untested"
grep -q 'missing_message_parameter_falls_back_without_changing_error_identity' "$src" \
  || fail "missing message parameter fallback is untested"

echo "STAGE7_COMPOSITION_REQUEST_OBSERVABILITY_OK"
