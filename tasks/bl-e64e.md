+++
title = "the Files tab has no viewer: clicking a file selects it but the preview is stacked under the listing instead of beside it"
created = 1787515080
updated = 1787515080
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
Reported by operator: in the Files tab for an agent you can click a file, but that does not let you view it. Wanted: the files list stays, and a viewer/editor pane opens to its right.

Today `src/files_view/render.rs::render_present` paints the listing and the preview inside ONE vertical ScrollArea, preview after a separator — so the selected file's bytes are below the fold of a long listing and the list scrolls away to read them.

Editing is a separate question: DESIGN I2 forbids a direct write inside a workspace (workspace state via lernie verbs only) and §11 states the tab is 'the agent worktree read-only'. Resolve in the ball: side-by-side viewer, and DESIGN amended to say why the pane is a viewer.