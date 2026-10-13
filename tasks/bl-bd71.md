+++
title = "the sign-in child inherits no display from the engine's service, so bz's browser launch dies — discover the session's display from its sockets when the service has none"
created = 1791865750
updated = 1791865825
claimant = "Surveying"
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["auth"]
+++
Operator ruling 2026-10-12: lernie's /login triggers the login and bz does it by launching a browser window; keep that flow. What breaks it: the yog user unit starts at default.target before the graphical session exports DISPLAY/WAYLAND_DISPLAY into the user manager, so the service — and every bz --login child it seats — has no display, and xdg-open dies with 'Error: no DISPLAY environment variable specified' into the login tail the seat paints. The service does have XDG_RUNTIME_DIR. Fix: when the engine seats the sign-in child and its own env names no display, derive one from the session's sockets — $XDG_RUNTIME_DIR/wayland-<n> → WAYLAND_DISPLAY, /tmp/.X11-unix/X<n> → DISPLAY=:<n> — and hand both to the child; a box with neither socket hands nothing and the device-code path stays what it is. Also seat the unit After=graphical-session.target where one exists? No: headless boxes have none; the discovery covers both.