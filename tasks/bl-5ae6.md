+++
title = "release workflow hands the crates.io token to jobs that do not publish, pins mutable action tags, and publishes a moving ref instead of the tested SHA"
created = 1786677238
updated = 1786677238
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