+++
title = "wall brazen config: derive the claude-cli user-agent with header_commands once yog links a brazen that has the key; and stop keeping four copies of the row table"
created = 1794896911
updated = 1794896911
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Source: ops bl-048a, brazen bl-0839 (main 2ede757, unreleased; first release carrying it will be > 0.0.20).

brazen gained a provider-row key: header_commands = { name = argv }. Trimmed stdout of argv is the header value per request; the same-named beta_headers entry is the fallback when the command yields nothing. Spec: brazen specs/config.md 4.3.2.

BLOCKER, in order:
1. brazen must publish a release containing header_commands.
2. yog (Cargo.toml: brazen = '=0.0.19') and litany (same pin, one brazen must resolve) step to it. Until then a wall config carrying the key is a MalformedFile for yog's linked brazen (deny_unknown_fields on the row) and every seat chat dies.
3. Then add to the claude-session-direct row in ~/.config/brazen/config.toml and world/walls/{dev,lab,ops}/brazen/config.toml, as a row key ABOVE [provider.oauth]:
   header_commands = { user-agent = ["sh", "-c", 'v=$(claude --version) && echo "claude-cli/${v%% *} (external, sdk-cli)"'] }
   Keep the static user-agent in beta_headers as the fallback. Check claude is on PATH where the seat's brazen runs; if not, the fallback is what is sent.
   scripts/drive/wall.sh seed_wall_config copies ~/.config/brazen/config.toml into a wall, so the host file must not gain the key before step 2 either.

Second defect (Omegas, 2026-10-29): the row table exists as four files that had already drifted (walls lacked the spud provider). One fact, four copies.