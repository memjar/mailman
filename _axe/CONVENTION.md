# Axe-spec — the AXE convention for living specs

A language- and tool-agnostic way to keep current truth, reviewable proposals
and an honest history next to the code. No tooling: you create and check the
structure by reading these rules.

The root is this repository, marked by the `_axe/` directory.

## Layout

- `_axe/spec/<capability>.md` — current truth, one file per capability. No
  history, no rationale: what the software does today.
- `_axe/changes/<change-id>/` — in-flight proposals, kebab-case ids.
- `_axe/log/YYYY-MM-DD-<change-id>.md` — dated history. Entries are immutable.

## A change

`_axe/changes/<change-id>/` holds:

- `proposal.md` — *why* and *what*. Frontmatter: `axe: change`, `id`,
  `status: active`, `created`.
- `tasks.md` — the steps, checked off as they land.
- `delta.md` — the exact edits to `spec/`, under literal headings
  `ADDED`, `MODIFIED`, `REMOVED`.

## Landing a change

1. Fold the delta into `_axe/spec/<capability>.md` so the spec always reflects
   current truth: append ADDED, replace MODIFIED, delete REMOVED.
2. Append `_axe/log/YYYY-MM-DD-<change-id>.md` saying what landed and which
   capabilities moved.
3. Delete the change directory. The log entry is the record.

## Checking

A root is valid when: `_axe/` is discoverable, `spec/ changes/ log/` exist,
every file carries a valid `axe:` type, each change has `proposal.md` and
`tasks.md`, ids are kebab-case, delta headings are literal, and every landed
change has a log entry. Everything else — prose, naming, ordering, extra
files — is free.
