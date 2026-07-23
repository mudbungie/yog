+++
title = "W8 embed balls: typed store reads in-process, mutations via yog bl multiplex"
created = 1784784298
updated = 1784784298
parent = "bl-b5d1"
priority = 2
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
DESIGN §16.7 W8. Needs W12 + U-balls (balls repo: promote Catalog/Entry/task_json pub + release). BlRunner prod impl goes in-process over the promoted read surface; mutating verbs spawn yog bl <verb> via multiplex.