# Capabilities — current API and usage

Each capability has **one canonical `README.md`** documenting what is implemented, how an application uses it, and current limits. A capability may span multiple crates; **do not create a folder for every crate**.

| Capability | Current implementation |
| --- | --- |
| [Application and identity](application/README.md) | `audiacore-core` |
| [Configuration](configuration/README.md) | `audiacore-config` |
| [Errors, messages and templates](errors/README.md) | `audiacore-errors`, `audiacore-template`, `audiacore-error-catalog` |
| [Logging and tracing](logging/README.md) | `audiacore-logging` |
| [Sensitive values](sensitive/README.md) | `audiacore-sensitive` |
| [Host access](host/README.md) | `audiacore-host`, `audiacore-host-native` |
| [Messaging and events](messaging/README.md) | `audiacore-events` (partial future messaging model) |
| [Reconciliation](reconciliation/README.md) | `audiacore-reconcile` |
| [Managed Content](managed-content/README.md) | `audiacore-managed-content` |
| [Workflow](workflow/README.md) | `audiacore-workflow` |
| [Time](time/README.md) | `audiacore-time` |

Capabilities without implemented APIs (for example durable broker delivery) remain in the [target state](../architecture/target-state.md), **not as empty folders**.

## Keep documentation small

- **Current use:** only `<capability>/README.md` is the public/current contract guide; describe tested API, minimal example or flow, guarantees, limitations, and source/test links. Do not place speculative designs here.
- **Work in progress:** add `<capability>/work/<topic>.md` only for a real investigation or active decision. **Never prefix or suffix a filename with a date. Edit or append to the same topic file as research progresses.** Include **Status**, **Question**, **Evidence**, **Options**, **Decision** (or `Pending`), **Validation**, and **Next action**. One living note per investigation, not daily progress logs.
- **Accepted decisions:** summarize the durable result in the capability guide (API behavior) or [global architecture](../architecture/core-capabilities.md) (cross-capability boundaries); link to its detailed evidence. Do not duplicate the rationale in multiple current pages.
- **Archive:** when research is closed and no longer useful for active work, move it to `docs/archive/<capability>/<topic>.md` (create only when needed), preserving a stable descriptive filename. Leave a short link from an accepted decision or history index if discoverability matters. Archives are not linked from the top-level current-use navigation.
- **Dates:** if relevant, record decisions or evidence dates **inside** the topic file, not in its path. Do not create a new file for each date or iteration.
- **History:** Git commits/PRs and [revalidation history](../architecture/revalidation.md) remain authoritative for chronology. Do not copy entire historical stage files into each capability.
- **Folders:** do not create `notes/`, `decisions/`, `backlog/`, `research/`, `archive/` and `drafts/` in every capability. Use the single optional `work/` directory.
- **Review:** every capability code change should check its corresponding current-use page. Retire or archive completed work notes rather than letting `work/` become a second roadmap.

A code crate and a documentation capability are not required to have identical names or 1:1 membership.
