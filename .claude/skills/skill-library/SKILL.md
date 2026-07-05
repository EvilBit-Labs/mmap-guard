---
name: skill-library
description: Router to ECC LIBRARY skills for this Rust library crate. Use when a task needs an off-stack ECC skill that is turned OFF by default (any language other than Rust, frontend/web, database, cloud/infra, networking, ML, agent-harness meta, domain/business ops). Names the relevant skill so it can be invoked directly with the Skill tool.
---

# Skill Library (mmap-guard)

This repo is a **pure Rust library crate** (the safe `mmap` boundary). Only a lean
DAILY set of ECC skills loads by default; everything else is turned OFF in
`.claude/settings.local.json` to keep context clean.

OFF does **not** mean deleted. Every skill below is still installed and can be
invoked on demand with the `Skill` tool (e.g. `Skill(skill="ecc:docker-patterns")`).
This router exists so those skills remain discoverable by keyword.

## DAILY (already loaded — no router needed)

- `rust-patterns`, `rust-testing` — the crate itself
- `git-workflow`, `github-ops` — commits, PRs, mergify, release-plz, `gh`
- `security-review` — the unsafe boundary this crate encapsulates

## LIBRARY — invoke on demand by keyword

### Rust adjacent (ordered by relevance to *this* crate)
- **memory safety / the unsafe boundary** → `systems-programming:memory-safety-patterns` — the crate's whole reason to exist; reach for it before touching the single `unsafe` block in `src/map.rs`
- **error handling on the failure paths** → `ecc:error-handling` — the value proposition is returning `io::Error` *gracefully* instead of crashing: empty file, concurrent truncation (SIGBUS), permission denied, advisory-lock `WouldBlock`
- **TDD discipline** → `ecc:tdd-workflow` — the repo mandates write-tests-first; the proptest suites and `fuzz/` targets that harden the boundary live under the DAILY `rust-testing`
- **security audit / bounty** → `ecc:security-bounty-hunter`, `ecc:security-scan`, `ecc:safety-guard`
- **benchmark / perf** → `ecc:benchmark`, `ecc:benchmark-methodology`, `ecc:latency-critical-systems`, `ecc:data-throughput-accelerator`
- **architecture decisions** → `ecc:architecture-decision-records`
- **codebase onboarding / tour** → `ecc:codebase-onboarding`, `ecc:code-tour`, `ecc:repo-scan`
- **coding standards** → `ecc:coding-standards` (marginal — the pedantic/nursery clippy config in `Cargo.toml` already enforces most of this in-repo)

> No `serde`, `clap`, `tokio`, `thiserror`, or async in this crate (deps are just
> `memmap2` + `fs4`), so the Rust *library* skills for those are noise here.

### Other languages (not present here, but installed)
python, django, fastapi, laravel/php, golang, java/springboot/quarkus, kotlin,
swift/swiftui, cpp, csharp/dotnet, fsharp, perl, ruby, react/vue/nuxt/angular,
dart/flutter → `ecc:<lang>-patterns`, `ecc:<lang>-testing`, `ecc:<lang>-security`.

### Web / frontend / design
`ecc:frontend-patterns`, `ecc:frontend-a11y`, `ecc:design-system`,
`ecc:motion-foundations` / `ecc:motion-advanced`, `ecc:accessibility`,
`ecc:make-interfaces-feel-better`, `ecc:taste`.

### Data / infra / cloud
`ecc:postgres-patterns`, `ecc:mysql-patterns`, `ecc:redis-patterns`,
`ecc:mongodb`, `ecc:prisma-patterns`, `ecc:database-migrations`,
`ecc:docker-patterns`, `ecc:kubernetes-patterns`, `ecc:deployment-patterns`,
`ecc:clickhouse-io`.

### Networking / homelab
`ecc:cisco-ios-patterns`, `ecc:netmiko-ssh-automation`,
`ecc:network-bgp-diagnostics`, `ecc:network-config-validation`,
`ecc:homelab-*`.

### Agent / harness / meta (ECC internals)
`ecc:configure-ecc`, `ecc:ecc-guide`, `ecc:skill-scout`, `ecc:skill-stocktake`,
`ecc:agent-sort`, `ecc:continuous-learning-v2`, `ecc:context-budget`,
`ecc:strategic-compact`, `ecc:eval-harness`, `ecc:autonomous-loops`.

### Domain / business ops
healthcare, finance/billing, logistics/ito/carrier, marketing/brand/seo,
scientific databases, blockchain/defi, video/media — search the OFF list in
`.claude/settings.local.json` and invoke `ecc:<name>` directly.

## Re-promoting a skill to DAILY

If a LIBRARY skill becomes routine, flip its value from `"off"` to `"on"`
(or remove the entry) in `.claude/settings.local.json` → `skillOverrides`.
