+++
title = "wire the missing OS confinement backend for unattended agents"
created = 1786843679
updated = 1786846056
claimant = "Confine-bca4"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["missing", "security", "confinement", "agentic"]
+++
The architecture explicitly records an unimplemented safety capability:

`yog wires no confinement layer today, so a workspace declaring confinement: required refuses every birth, by name.`

It further states:

`Not confinement: the ambient PATH, network, and the deliberately-shared brazen credentials stay reachable by an allowed invocation, and rule classification bounds accident, not adversarial evasion.`

The existing refusal is correct fail-closed behavior, but it leaves no usable confined mode. An unattended workspace must either run with host files, processes, environment, and network reachable, or declare `confinement: required` and become unable to start any agent.

History-wide task searches find only closed bl-0cea, the capability-boundary design. That task explicitly deferred `Optional OS confinement as a later, platform-explicit layer`; no live implementation task carries the missing layer.

Required outcome: wire one platform-explicit confinement backend through the reserved lernie spawn seam, derive availability rather than storing it, and spend the existing `confinement: required` signal as the fail-closed gate. Specify and test the support boundary for filesystem writes, process access, environment, and network. Unsupported platforms must retain the present explicit refusal; there must be no silent fallback.