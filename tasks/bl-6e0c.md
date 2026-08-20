+++
title = "help serve still describes the retired pre-self-provisioning wire behavior"
created = 1787206336
updated = 1787207206
claimant = "Zircons-Misc"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["bug", "cli", "docs", "wire"]
+++
`yog help serve` says:

> “the listener is up only where an operator has provisioned certificates (`make wire-certs`), and silently absent otherwise.”

Current engine boot self-mints loopback certificate material when absent, and listener/provisioning failures are visible refusals. `yog wire-certs --help` already describes the current behavior; the serve page is stale after `bl-dc14`.

Update the serve contract to state what boot provisions, when the explicit verb is needed, and how failure is surfaced. Add parity coverage against the provisioning authority so the two help pages cannot drift independently.