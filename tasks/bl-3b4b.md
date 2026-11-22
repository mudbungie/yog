+++
title = "a machine mid-job vanishes from provider listings and refuses named reads: busy is told from asleep by the job the mailbox holds, and a busy machine answers from its last known rows"
created = 1795322339
updated = 1795322339
claimant = "Rabid-busy"
priority = 2
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
+++
A registered machine that runs model calls holds no parked read while it works, and the listings and the named model read test only for a parked read. So the machine drops out of providers and catalog, and a models read naming it is refused as if it were asleep, for exactly as long as it is doing something.

Fix without a wire change: the mailbox already holds the job it handed that machine. A machine is busy while such a job is unfinished, its asker still listens and its run has not gone silent - the same rules that already end a run. A listing keeps a busy machine's rows from the last answer that machine gave to the same read (one replica, in the mailbox, replaced by every fresh answer, served only while busy); a named read answers the same way, and says busy rather than asleep when nothing is known. REMOTE 5.3, 5.4 and 5.7 are amended to match.