+++
title = "a conversation's procedure runs are a reply: per run the role, the message or tool result that fired it, state, epitaph, verdict token, landed file content, tokens and cost"
created = 1792729503
updated = 1792729503
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
The seat paints a procedure as a marker on the message that fired it (lernie bl-454d), which needs the runs as facts. All of it is held on disk today: dispatch commit subject, landing commit 'landing: <file> [<child>]', the child's step usage. A fold, not storage. Whether it rides Reply::Transcript as an entry kind or a query of its own is this ball's design question; prefer the transcript, since a run belongs at a position in it.