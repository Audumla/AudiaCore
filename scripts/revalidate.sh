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

# Clean-room repository: reject known compatibility/legacy debris and duplicate
# provider-specific instruction surfaces. Git history is the archive.
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

# Stage-aware validation. Later stages extend this script rather than creating
# competing validation entry points.
if [[ -f Cargo.toml ]]; then
  cargo fmt --all -- --check
  cargo clippy --workspace --all-targets --locked -- -D warnings
  cargo test --workspace --locked
fi

echo "REPOSITORY_DISCIPLINE_OK"
if [[ -f Cargo.toml ]]; then
  echo "RUST_WORKSPACE_OK"
fi
echo "AUDIACORE_REVALIDATION_OK"
