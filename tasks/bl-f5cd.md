+++
title = "REMOTE 5.8 in force: a host that sends holding on its invocations read is pushed every job over one open read, with a heartbeat; old hosts unchanged"
created = 1795581082
updated = 1795581082
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["feature"]
+++
Rewrite REMOTE 5.8 from proposal to the wire in force and build the server half. A host stating holding (a list, possibly empty) on its invocations read keeps that read open indefinitely and is written one reply frame per job as it is queued for its certificate; an empty heartbeat frame every 30 s; holding reconciles leases on (re)connect; a new pushing read supersedes the old; an ask nobody listens for is not handed out; a read without holding is today's byte for byte. Edition, not a PROTOCOL bump. room is deleted from the design.