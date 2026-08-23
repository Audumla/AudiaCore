#!/usr/bin/env python3
"""Enforce AudiaCore's direct Rust dependency admission policy.

Third-party dependencies are approved once in the workspace root and member
crates inherit them with `workspace = true`. Local path dependencies must point
to declared workspace members. The check covers normal, dev, build, and
target-specific dependency tables.
"""

from __future__ import annotations

import sys
import tomllib
from pathlib import Path
from typing import Any, Iterator

ROOT = Path(__file__).resolve().parents[1]
DEPENDENCY_SECTIONS = ("dependencies", "dev-dependencies", "build-dependencies")


def fail(message: str) -> None:
    print(f"DEPENDENCY_ADMISSION_FAIL: {message}", file=sys.stderr)
    raise SystemExit(1)


def load_manifest(path: Path) -> dict[str, Any]:
    try:
        return tomllib.loads(path.read_text(encoding="utf-8"))
    except (OSError, tomllib.TOMLDecodeError) as error:
        fail(f"cannot parse {path.relative_to(ROOT)}: {error}")


def dependency_tables(manifest: dict[str, Any]) -> Iterator[tuple[str, dict[str, Any]]]:
    for section in DEPENDENCY_SECTIONS:
        table = manifest.get(section)
        if isinstance(table, dict):
            yield section, table

    targets = manifest.get("target", {})
    if not isinstance(targets, dict):
        fail("manifest [target] entry is not a table")
    for target_name, target in targets.items():
        if not isinstance(target, dict):
            fail(f"manifest target {target_name!r} is not a table")
        for section in DEPENDENCY_SECTIONS:
            table = target.get(section)
            if isinstance(table, dict):
                yield f"target.{target_name}.{section}", table


def main() -> None:
    root_manifest = load_manifest(ROOT / "Cargo.toml")
    workspace = root_manifest.get("workspace")
    if not isinstance(workspace, dict):
        fail("root Cargo.toml has no [workspace] table")

    members = workspace.get("members")
    if not isinstance(members, list) or not all(isinstance(member, str) for member in members):
        fail("workspace.members must be an explicit string list")

    approved = workspace.get("dependencies")
    if not isinstance(approved, dict):
        fail("root Cargo.toml has no [workspace.dependencies] approval table")

    member_dirs = {(ROOT / member).resolve() for member in members}
    problems: list[str] = []

    for member in members:
        manifest_path = ROOT / member / "Cargo.toml"
        if not manifest_path.is_file():
            problems.append(f"{member}: missing Cargo.toml")
            continue

        manifest = load_manifest(manifest_path)
        for section, dependencies in dependency_tables(manifest):
            for name, specification in dependencies.items():
                location = f"{manifest_path.relative_to(ROOT)} [{section}] {name}"

                if isinstance(specification, str):
                    problems.append(
                        f"{location}: direct third-party version must inherit an approved workspace dependency"
                    )
                    continue

                if not isinstance(specification, dict):
                    problems.append(f"{location}: unsupported dependency declaration")
                    continue

                if specification.get("workspace") is True:
                    if name not in approved:
                        problems.append(
                            f"{location}: workspace dependency is not approved in [workspace.dependencies]"
                        )
                    forbidden = {"path", "git", "registry", "version"}.intersection(specification)
                    if forbidden:
                        problems.append(
                            f"{location}: inherited dependency also declares {', '.join(sorted(forbidden))}"
                        )
                    continue

                path = specification.get("path")
                if isinstance(path, str):
                    resolved = (manifest_path.parent / path).resolve()
                    if resolved not in member_dirs:
                        problems.append(
                            f"{location}: path dependency resolves outside declared workspace members: {path}"
                        )
                    continue

                problems.append(
                    f"{location}: registry/git dependency must be approved at workspace root and use workspace = true"
                )

    if problems:
        print("DEPENDENCY_ADMISSION_FAIL: direct dependency policy violations", file=sys.stderr)
        for problem in problems:
            print(f"  - {problem}", file=sys.stderr)
        raise SystemExit(1)

    print("DIRECT_DEPENDENCY_ADMISSION_OK")


if __name__ == "__main__":
    main()
