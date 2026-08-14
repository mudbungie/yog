+++
title = "release workflow hands the crates.io token to jobs that do not publish, pins mutable action tags, and publishes a moving ref instead of the tested SHA"
created = 1786677238
updated = 1786677374
claimant = "Marinara"
priority = 3
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
tags = ["publication"]
+++
Source: publication audit follow-up 2026-08-13 (item 2), snapshot yog `e758814`.

Confirmed finding: **publishing credential exposed to workflow code, not
publicly disclosed**. No publishing credential was found committed or printed
unmasked. All 144 retained successful Release-plz log archives were scanned:
9,751,767 decompressed bytes, 3,299 redaction markers, zero token-shaped
values, zero unmasked credential assignments, zero unmasked Authorization
values. That supports "not publicly disclosed"; it cannot prove what
third-party action code did over the network.

Current `.github/workflows/release-plz.yml`:

```yaml
- uses: release-plz/action@v0.5
  env:
    GITHUB_TOKEN: ${{ secrets.RELEASE_PLZ_TOKEN || secrets.GITHUB_TOKEN }}
    CARGO_REGISTRY_TOKEN: ${{ secrets.CARGO_REGISTRY_TOKEN }}
```

The repository has one Actions secret, `CARGO_REGISTRY_TOKEN`;
`RELEASE_PLZ_TOKEN` is absent. The crates token is passed to BOTH the release
job and the PR-maintenance job; the latter only opens/updates a PR and needs no
publish authority. All actions use mutable tags, not full commit SHAs. The
workflow grants `contents: write` and `pull-requests: write` globally.

The privileged release job checks out:

```yaml
ref: ${{ github.event.workflow_run.head_branch || github.ref_name }}
```

That is moving `main`, not the exact successful
`github.event.workflow_run.head_sha` — code B may publish after CI tested code
A. Manual dispatch can also enter the publish job with no CI verdict. The
binary job interpolates dispatch input straight into generated Bash:

```bash
tag="${{ github.event.inputs.binaries_tag }}"
```

Required boundary:

1. remove the crates token from PR maintenance;
2. environment-protect the publish secret;
3. pin every action to a verified full commit SHA;
4. scope permissions per job, not globally;
5. publish only the exact tested SHA (`workflow_run.head_sha`);
6. separate manual binary backfill from publish;
7. pass tag input through an environment variable, never direct expression
   interpolation in shell;
8. disable checkout credential persistence unless a step needs it.

GitHub's own guidance for 1-8:
- https://docs.github.com/en/actions/reference/security/secure-use
- https://docs.github.com/en/actions/concepts/security/script-injections

Verify each claim against the tree before editing; the audit is a snapshot.
Do not rotate or touch live secrets — that is an operator action.

---

Verified every audit claim against the tree at e758814 before editing. All seven
findings hold exactly as filed: both release-plz jobs got CARGO_REGISTRY_TOKEN;
`gh secret list` shows exactly one Actions secret (CARGO_REGISTRY_TOKEN, created
2026-07-26) and no RELEASE_PLZ_TOKEN; `gh api repos/mudbungie/yog/environments`
returns an empty list, so nothing was environment-protected; every action rode a
mutable tag; the release job checked out `head_branch || ref_name`; a dispatch
could enter the publish job; the binaries job spliced `${{ github.event.inputs.
binaries_tag }}` into bash.

Three corrections/additions to the snapshot:

1. "grants contents: write and pull-requests: write globally" is true, but not
   uniformly — `prune-release-branches` already carried a job-level
   `permissions:` block. It was the only job that did.
2. The manual binary backfill the audit treats as a live publish-adjacent path
   was DEAD CODE. `release-binaries` has `needs: release-plz-release`, and a job
   whose dependency is SKIPPED is itself skipped unless its `if` calls a status
   function; on `workflow_dispatch` the release job was skipped, so the
   `github.event_name == 'workflow_dispatch'` clause in the binaries `if` never
   got the chance to evaluate. Point 6 was therefore separating a path that had
   never run. The fix keeps one binaries job with `!cancelled()` in the
   condition, which makes backfill work for the first time, and removes
   `workflow_dispatch` from the release job entirely so dispatch cannot publish.
3. Not in the audit: README's "Publishing" section said publishing is "a
   deliberate human decision, never a side effect of CI or delivery" — false
   since this workflow landed. Rewritten to describe both paths and point at the
   boundary.

Also not in the audit, and it blocks verification: EVERY GitHub Actions run on
this repo currently fails before a single step executes. Annotation on the most
recent run (31678203160, 2026-08-13): "[redacted: account-level scheduling annotation]
Please check the 'Billing & plans' section in your settings". CI and Release-plz
have both been failing this way all of 2026-08-13, so the hardened workflow has
not been exercised by a live run and cannot be until that clears.

OPERATOR ACTIONS (deliberately not taken — read-only against GitHub):

* Create the Actions environment named `publish` (Settings -> Environments),
  restrict its deployment branches to `main`, and optionally add yourself as a
  required reviewer. Then move CARGO_REGISTRY_TOKEN into that environment as an
  environment secret and DELETE the repo-level secret of the same name. The
  workflow works either way — a repo secret still resolves for a job with an
  environment — so until the move, boundary rule 2 is declared but not enforced.
* Consider rotating the crates.io token. Nothing shows it disclosed, but it has
  been handed since 2026-07-26 to a job with no publish business and to
  third-party action code running under mutable tags; rotation is cheap and
  ends that exposure window definitively.
* Clear the GitHub Actions account-level block, then dispatch Release-plz once with a known tag
  (e.g. v0.0.1) to confirm the backfill path now runs.
* Unchanged and still required: Settings -> Actions -> General -> "Allow GitHub
  Actions to create and approve pull requests".
