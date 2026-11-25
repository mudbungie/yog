+++
title = "REMOTE 5.8: an id also leaves holding when its run ended with no final post to make"
created = 1795582327
updated = 1795582328
claimant = "Rabid"
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["docs"]
+++
Found while building the host half (thrall bl-a7f0). 5.8 names one way an id leaves holding: its final complete is answered. A run can end with no final post at all: a partial post refused (5.7 ends the slot in doubt) or swallowed by the wire (5.7: the child is stopped). Such an id would be listed forever. State that holding is the jobs a host still has something to post for.

Tested by: the doc text; the host's own test of a dropped partial lives in thrall.