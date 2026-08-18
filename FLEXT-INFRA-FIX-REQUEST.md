# flext-infra fix request — branch `0.12.0-dev`

Repo: `https://github.com/flext-sh/flext-infra` · Branch: `0.12.0-dev`
Verified against: `origin/0.12.0-dev` = `c5a5f2ec6ca5380c23a0dec7ca53e1dce727b7cf`
Reported from: `mcb` @ `develop`, after `uv lock --upgrade-package flext-infra` + `uv sync`

Both defects are in the generated `.pre-commit-config.yaml`. mcb consumes
flext-infra as a pinned git rev from GitHub, so neither can be fixed in mcb.

---

## Defect 1 (P0) — pre-commit `entry` is unrunnable, blocks every commit

### Symptom in any consuming project

```text
make check...............................................................Failed
- hook id: flext-pre-commit-check
- exit code: 1

Executable `CI=Y` not found
```

Reproduce (mcb, current `origin/0.12.0-dev`):

```bash
.venv/bin/python -m pre_commit run flext-pre-commit-check --hook-stage pre-commit --all-files
```

### Rendered output today

```yaml
- id: flext-pre-commit-check
  name: make check
  language: system
  entry: >-
    CI=Y make check
```

### Root cause

`pre-commit` with `language: system` executes `entry` as an **argv vector**, never
through a shell. A shell would read `CI=Y make check` as "assign CI, then run make";
`execvp` instead takes the first token as the **program name** and searches for a
binary literally named `CI=Y`.

The `pre_push` branch of the same template never hit this because it already wraps
in `bash -eu -o pipefail -c '...'` (for `unset $(git rev-parse --local-env-vars)`),
and that wrapper incidentally makes a leading assignment legal. So the bug is
specific to the `pre_commit` branch and only appeared once `CI=Y` was added there.

### File

`src/flext_infra/templates/project/base/.pre-commit-config.yaml.j2`

The `pre_commit` loop's `entry` line currently begins with:

```jinja
{% if step.verb == "check" %}{{ make.ci.variable }}={{ make.ci.value }} {% endif %}...
```

**Required change** — state the variable through `env`, which is the portable way
to set a variable for a command that runs without a shell, and apply it to every
step rather than only `check`:

```jinja
env {{ make.ci.variable }}={{ make.ci.value }} ...
```

**Acceptance**: in a consuming project, `pre-commit run flext-pre-commit-check
--hook-stage pre-commit` exits 0, and the rendered entry reads
`env CI=Y make check`.

---

## Defect 2 (P1) — pre-push states no CI token, so the tier runs in the wrong ternary arm

### Rendered output today

```yaml
- id: flext-pre-push-check
  entry: >-
    bash -eu -o pipefail -c 'unset $(git rev-parse --local-env-vars); make check'
```

No `CI` token at all.

### Why that is wrong

`config/codegen.yaml` already documents the contract (RULING 1):

> CI is ternary (Y/N/absent). CI=Y revokes pytest; CI=N runs full pytest+coverage;
> absent=testmon incremental.

With no token the pre-push tier runs the **absent** arm (testmon incremental)
instead of the **N** arm (full suite + coverage + every blocking gate) that a
push gate requires. Worse, a git hook inherits the invoking environment, so a
caller that already exported `CI=Y` leaks it into pre-push and revokes exactly
the gates a push must not skip (lint/format/pyrefly per 9b604d43, pytest per
mro-v4p5). Unsetting is not the answer either, because pre-commit legitimately
wants `CI=Y`. Each tier must **declare** its arm.

### Required changes

1. `config/codegen.yaml`, in the `make.ci` block that already holds
   `variable: CI` / `value: Y`, add the local arm so no surface spells a bare
   `"N"` literal:

   ```yaml
   local_value: N
   ```

2. The matching field on the model that backs that block
   (`src/flext_infra/_models/config.py`, the spec exposing `make.ci.*`), so the
   template can read `make.ci.local_value`.

3. `src/flext_infra/templates/project/base/.pre-commit-config.yaml.j2`, inside the
   `pre_push` loop's existing `bash -c` wrapper, state the token after the `unset`:

   ```jinja
   bash -eu -o pipefail -c 'unset $(git rev-parse --local-env-vars); {{ make.ci.variable }}={{ make.ci.local_value }} ... make {{ step.verb }} ...'
   ```

   No `env` is needed here because this branch already owns a shell.

**Acceptance**: the rendered pre-push entry reads
`bash -eu -o pipefail -c 'unset $(git rev-parse --local-env-vars); CI=N make check'`,
and a push runs the full pytest+coverage suite with every blocking gate.

---

## Note

Local commits `b9079aa5f` and `c81cc5aae` in the working checkout at
`/home/marlonsc/flext/flext-infra` already implement exactly these two changes
and match this specification. They have not reached `origin/0.12.0-dev`. That
checkout also carries a third party's uncommitted work in
`src/flext_infra/deps/phases/ensure_pyright.py`, which this session left untouched.

## Impact while unfixed

Commits in mcb only succeed by regenerating `.pre-commit-config.yaml` from the
stale template, which restores the older monolithic hook layout. Once flext-infra
lands the fix, mcb adopts it with `uv lock --upgrade-package flext-infra` +
`uv sync` + `make gen WHAT=all APPLY=Y`, with no hand-edit to any generated file.
