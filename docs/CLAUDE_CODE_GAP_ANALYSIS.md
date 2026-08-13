# Claude Code gap analysis

**Status:** evidence review reconciled through yog `9b0b2f4`, balls `0.5.9`,
brazen `0.0.5`, and lernie `0.0.3`; Claude Code source snapshot
`06d29efd02547a586a33cab60e8acf3dba2997e8` (dated 2026-03-31). The local
installed CLI was `claude 2.1.220` when its public `--help` surface was checked.
The GitHub repository is an unofficial third-party source snapshot, not an
Anthropic release; provenance and completeness are unverified.

## Verdict

Claude Code is presently the stronger **single-repository coding cockpit**.
The likely cause is not its command count. It closes the high-frequency loop:

> establish the project and its rules → choose a narrow coding operation →
> execute it safely and concurrently → bound what returns to context → show the
> user the diff, decision, failure, and recovery in place.

Yog presently is a **durable cross-project control plane around a thinner
coding loop**. Its real advantages are disk-derived truth, committed agent
history, durable asynchronous children, provider choice, and balls' task /
worktree / blocker / close semantics. Its intended task-first fleet, fork/fan
adjudication, cost ceilings / portfolio rollup, and headless administration
are not current advantages: `docs/VISION.md` records them as gaps or future
rungs.

The source supports causal hypotheses, not a measured win. The snapshot has
three commits, 1,904 tracked files, 1,884 TypeScript/TSX files and 512,664
TypeScript/TSX lines, but no package manifest, lockfile, or named test files. It
cannot be built or benchmarked as checked out. Many candidate tools are feature
gated. No claim below says Claude Code has a higher solve rate or lower wall
time until `bl-36fa` makes a controlled run possible.

## Comparison boundary

Comparing Claude Code only with `src/yog` is a category error. The unit under
test is the four-layer suite:

| Layer | Present responsibility |
|---|---|
| balls | durable tasks, blockers, claims, project worktrees, squash delivery |
| brazen | provider-neutral request/event wire, usage, Anthropic prompt caching |
| lernie | agent branches, context assembly, tools, inbox, children, compaction |
| yog | policy, observation, operator actions, nested-world composition |

Claude Code covers analogous concerns in one vertically integrated process,
though it has no balls-equivalent blocker, claim, or atomic-close substrate.
That coupling is both its current advantage and its long-term cost.

## Why Claude Code is likely better today

### 1. Project grounding is mechanism, not a reminder

Claude Code snapshots branch, status and recent commits and discovers its
`CLAUDE.md` / `.claude/CLAUDE.md` / rules hierarchy before work
(`src/context.ts:36-103,152-187`; `src/utils/claudemd.ts:790-934`). It does not
normally auto-load this repository's `AGENTS.md`; the advantage is a context
ingestion mechanism, not a magic filename.

Yog has a deeper defect than missing instruction discovery. A ball-bound root
spans two Git worlds:

1. balls owns the external project worktree that `bl close` delivers;
2. yog writes that absolute path into the goal and starts the initial lernie
   process there (`src/start/goal.rs:110-118,140-158`);
3. pinned lernie runs every tool in its separate
   `<workspace>/agents/<id>` worktree
   (`lernie-0.0.3/src/prompt/tool/spawn.rs:159-165,199-214`);
4. lernie's tool commit stages only that agent worktree
   (`lernie-0.0.3/src/prompt/dispatch/transcript.rs:120-146`).

The model must remember prose and use absolute paths. Project edits made there
are outside lernie's commit-per-side-effect history, child inheritance, bundle,
replay, and sibling refs. Lernie main has since added a persistent `cd` mark,
but its architecture correctly calls external edits off-record and the mark
does not cross forks. A bare `--cwd` would hide, not resolve, the split.

This is the largest correctness gap because it contradicts yog's own promise:

> “If something must happen, a mechanism introduces it. Instructions decay
> against a growing context; gates, spawn-time seeding, and blocker edges do
> not. Protocol over prompt.” (`docs/VISION.md`, §1)

### 2. Coding actions are narrow and validated

Claude Code's candidate base pool includes ranged/multimodal Read,
Grep/Glob, Edit/Write, Bash, Agent, skills and optional LSP/MCP/worktree tools
(`src/tools.ts:193-250`). Simple mode still retains Bash, Read and Edit
(`src/tools.ts:271-298`). Edit checks read-before-write/staleness, produces a
diff and feeds IDE/LSP hooks.

Pinned lernie ships five default tools: `bash`, `dispatch`, `load_skill`,
`message`, and whole-file `read_file`. Its external-tool seam is real, so the
architecture is extensible; the shipped coding path is nevertheless shell plus
a one-mebibyte whole-file reader. Shell quoting, search conventions, exact edit
matching and diff interpretation consume model steps that Claude Code turns
into typed contracts. `bl-ae6b` is deliberately one patch tool, not a clone of
the catalog.

### 3. Context pressure is governed continuously

Claude Code normally spills a large tool result to disk, returns a bounded
preview plus address, applies an aggregate result-message budget, and layers
microcompaction, collapse, proactive compaction and one guarded overflow
recovery (`src/utils/toolResultStorage.ts:108-183,267-333`;
`src/constants/toolLimits.ts:5-49`; `src/query.ts:365-465,1062-1182`). These
paths have exceptions, so “all output is hard bounded” would be false.

Lernie preserves raw `stdout`/`stderr` in diagnostic `output.json`, but commits
the complete model-facing result. `bash` is unbounded; `read_file` rejects over
one mebibyte but has no range input; transcript entries sit outside the body
budget. Default compaction is every 20 commits, not actual context pressure.
One chatty build can poison every later request before that checkpoint.
`bl-ffc5` then `bl-d5fa` establish the safe floor: raw authority remains, the
model receives an honest bounded projection.

### 4. Independent reads overlap

Claude Code classifies tool calls for concurrency, runs safe consecutive calls
with a default cap of ten, and buffers results in source order
(`src/services/tools/toolOrchestration.ts:8-12,19-82,84-176`). Its streaming
executor can begin complete tool blocks while the model stream continues.

Pinned lernie waits for the model response to settle, then executes sibling
tool calls sequentially and commits each result in emission order
(`lernie-0.0.3/src/prompt/dispatch/tool_step.rs:1-18,62-106`). That ordering is
simple, deterministic and crash-friendly. It also serializes independent
reads. Process and Git boundaries add fixed cost, but their magnitude is
unmeasured; replacing the durable loop before profiling would be cargo-cult
optimization. Parallel execution is intentionally not filed until effect
classification and `bl-36fa` exist.

### 5. Permissions let the operator grant more autonomy

Claude Code validates tool input and mediates allow/deny/ask before execution.
Its installed CLI exposes permission modes and tool allow/deny lists. Its OS
sandbox is a separate option and defaults **off**
(`src/utils/sandbox/sandbox-adapter.ts:459-476,528-546`), so default-on sandbox
is not an advantage claimed here.

Lernie enforces role-level tool names, but granted `bash` is host-authority
`sh -c`. Yog's nested world isolates substrate state; it deliberately leaves
host files, PATH, network, processes and brazen credentials reachable. Balls
worktrees recover many code mistakes, not an external command, secret read or
network side effect. `bl-0cea` designs an asynchronous capability boundary;
copying a modal REPL approval would deadlock unattended drones.

### 6. The correction loop is visible and close at hand

The installed Claude Code CLI verifies current public surfaces for headless
streaming, budgets, permissions, resume/fork, background agents, Git
worktrees, MCP, IDE, remote control and doctor. The snapshot implements inline
progress/diffs, cancellation, prompt queuing, search/typeahead, permission
previews and conversation/file rewind. Rewind is not total: its UI warns that
manual/Bash edits are unaffected (`src/components/MessageSelector.tsx:345-349`).

Yog's restart story is stronger—selection reopens durable disk state—and its
raw forensic inspectors preserve exact bytes. But retrieval and correction are
slower today: work
diff is absent. (Two of this paragraph's three gaps have since closed: the
boundary made the action/query surface headless and typable — DESIGN §8.5 —
and bl-3c28 gave `Ctrl+F` a search across balls, workspaces, conversations and
transcripts, so the “no search surface exists” line it quoted is gone from
DESIGN.) Finished-turn rollups, the live in-flight strip and
per-conversation/per-ball spend attribution have landed. The right response is
search, commit-aware diff/history, portfolio cost/ceilings and headless
parity—not Claude Code's whole REPL.

## What yog already does well

Do not file fake gaps:

- **Streaming exists.** Brazen streams, lernie appends `response.json`, and yog
  renders the live tail (`src/git_tree/streaming.rs`; `src/transcript/mod.rs`).
- **Prompt caching exists.** Brazen automatically marks stable Anthropic
  segments (`brazen-0.0.5/src/protocol/anthropic/encode/cache.rs`). Claude Code
  has better visibility/stable tool ordering and conditional fork sharing.
- **Subagents exist.** Lernie dispatch returns immediately, gives each child a
  durable branch/worktree/address, and deposits the result later. Ordinary
  Claude Code agents also start fresh; full-context/cache-sharing fork is
  feature gated in the snapshot.
- **Claude Code tasks are not RAM-only.** Its sessions and task records are
  disk-backed. Balls is still categorically deeper: versioned shared backlog,
  blocker graph, claim/worktree occupancy, close gates and atomic delivery.
- **Disk truth is a product advantage.** Yog's “Disk is the app” invariant,
  deterministic two-instance convergence and owner-routed writes make crash
  recovery and forensic inspection unusually strong.
- **Provider neutrality is real.** Brazen permits model/provider experiments
  and avoids vendor lock-in. It cannot exploit every Claude-specific feature as
  quickly or as deeply.

## Closure order

| Order | Work | Task |
|---|---|---|
| P0 | Rule one authoritative project-work/delivery graph before cwd, diff, fan or fleet code | yog `bl-2b8c` |
| P0 | Make result shape honest, then bound model-facing output while retaining raw bytes | lernie `bl-ffc5` → `bl-d5fa` |
| P0 | Add generic pinned documents; then discover/freeze project rules visibly | lernie `bl-fb5c` → yog `bl-aa8b` |
| P0 | Design tool capabilities and durable asynchronous decisions before unattended fleet execution | yog `bl-0cea` |
| P0 | Extend the existing 50-task eval authority with wall/attempt/tool/usage metrics | lernie `bl-36fa` |
| P1 | Add one stale-safe, ambiguity-safe atomic patch tool | lernie `bl-ae6b` |
| P1 | Complete typed GUI/headless parity and fork exposure | yog `bl-8aab`; lernie `bl-a693` |
| P1 | Add derived global search/quick-open after the typed query boundary | yog `bl-3c28` |
| P1 | Implement project diff only after the two-Git ruling names its authority | yog `bl-3746` (blocked on `bl-2b8c`) |
| P2 | Design, rather than assume, an external MCP bridge | lernie `bl-3c76` |
| Cleanup | Delete yog's obsolete no-op worker-tool grant and its stale design | yog `bl-7fc8` |

Mandatory tests/docs/alignment gates were filed under `bl-fb5c`, `bl-36fa`,
`bl-d5fa`, `bl-ae6b`, and `bl-3c76`. Existing tasks cover headless control,
fleet cadence, fork/config exposure and output envelopes; no duplicates were
filed. Spend attribution (`bl-afc4`) and transcript rollup (`bl-1f21`) landed
during this review and are excluded from remaining work.

## Real tradeoffs

| Choice | Claude Code buys | Yog buys | Cost to preserve consciously |
|---|---|---|---|
| Vertical vendor integration vs neutral wire | prompt/tool/model co-design and earlier proprietary features | provider choice and comparative policy | neutrality will usually trail the best vendor-specific harness |
| Hot in-process loop vs process/Git seams | lower fixed latency and overlapping work | replay, ordered commits, crash isolation, independent lifetime | instrument first; optimize only measured seams |
| Wide structured toolset vs narrow external seam | fewer ambiguous actions and retries | less schema/security/UI/test surface | add deep primitives one at a time, each eval-proven |
| Automatic project context vs explicit frozen inputs | current rules with no operator step | reproducible task-bounded context | snapshot can go stale; dynamic loading can change invisibly; parent files can inject policy |
| Interactive permission prompts vs unattended drones | confidence and fine-grained consent | nonblocking throughput | “ask” must become a durable attention fact with an explicit safe default |
| Bounded projections vs complete context | smaller, faster subsequent requests | exact forensic bytes | retain raw output and an address; never silently discard decisive evidence |
| Session cockpit vs task control plane | immediate correction in one repository | durable multi-project backlog and delivery | yog should not become another terminal or make agents the durable unit |
| Remote/IDE/plugin breadth vs local one-binary world | reach and integration convenience | privacy, reproducibility and small trust surface | add connective tissue outside core only for a named deployment |
| Aggressive feature velocity vs strict Rust gates | faster surface expansion | containment, 100% coverage, smaller audit burden | yog will ship fewer features and must choose higher-leverage ones |

## Explicit non-goals

- Do not clone Claude Code's slash commands, onboarding wizard, terminal UI,
  marketplace, voice, Vim/keymap system, cloud bridge, teams or cron surface.
- Do not add opaque automatic memory. Freeze explicit instructions, tasks,
  skills and artifacts with provenance.
- Do not implement destructive partial rewind. Yog's commit-addressed fork is
  the sound history gesture.
- Do not put MCP, pricing, scheduler policy or tool catalogs into lower cores.
- Do not build parallel tool execution until tools declare effects and evals
  show the benefit; preserve emission-order commits.
- Do not replace lernie's durable process/commit loop from source-level latency
  inference. Measure it.
- Do not call Claude Code categorically safer because sandbox code exists; its
  default OS sandbox is off.
- Do not call roadmap rungs current yog advantages.

## Evidence index

Claude Code snapshot `06d29ef`:

- context and instruction discovery: `src/context.ts:36-103,152-187`,
  `src/utils/claudemd.ts:790-934`
- tool pool and cache-stable assembly: `src/tools.ts:193-250,325-366`
- safe concurrency: `src/services/tools/toolOrchestration.ts:8-176`,
  `src/services/tools/StreamingToolExecutor.ts:34-150`
- output spill/budget: `src/utils/toolResultStorage.ts:108-183,267-333`,
  `src/constants/toolLimits.ts:5-49`
- permission before call: `src/services/tools/toolExecution.ts:916-1029`,
  `src/hooks/toolPermission/PermissionContext.ts:45-125`
- sandbox default and fallback: `src/utils/sandbox/sandbox-adapter.ts:459-546`
- context recovery/retry: `src/query.ts:365-465,1062-1255`
- fresh vs context-inheriting agents: `src/tools/AgentTool/prompt.ts:255-272`,
  `src/tools/AgentTool/forkSubagent.ts:32-68`
- disk task records: `src/utils/tasks.ts:190-307`
- active-session diff/rewind: `src/components/diff/DiffDialog.tsx:23-96`,
  `src/components/MessageSelector.tsx:330-350`

Yog suite:

- product boundary and disk invariants: `docs/DESIGN.md` §0, §2
- task-first promise and present gap ledger: `docs/VISION.md` §§1-3
- current external project binding: `src/start/goal.rs:110-158`
- pinned tool cwd/result commits: lernie 0.0.3
  `src/prompt/tool/spawn.rs:159-214`,
  `src/prompt/dispatch/transcript.rs:120-146`
- sequential tools/full result: lernie 0.0.3
  `src/prompt/dispatch/tool_step.rs:1-121`
- whole-file cap: lernie 0.0.3
  `src/prompt/tool/builtin/read_file/mod.rs:8-24`
- durable child dispatch: lernie 0.0.3 `skills/dispatch/SKILL.md:8-58`
- automatic prompt caching: brazen 0.0.5
  `src/protocol/anthropic/encode/cache.rs:1-40,109-113`
