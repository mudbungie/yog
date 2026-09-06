+++
title = "litany's stop now always takes the children, so the boundary's stop_children field and the 'stop with its children' offer name no choice"
created = 1788675635
updated = 1788675635
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["usability-r2"]
+++
litany bl-3114 (76a89289) made `litany stop` walk every descendant unconditionally; `--stop-children` still parses and changes nothing. yog's boundary field `children` on the stop gesture, the attention row's offer text and lernie's spelling (bl-9fd1) now describe a distinction that no longer exists. Drop the field on the next PROTOCOL bump (or keep it accepted-and-ignored with the doc saying so), fix the offer text, and re-vendor the seats.