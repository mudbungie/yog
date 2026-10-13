+++
title = "sign-in from a seat opens a browser on the engine's box and dies without DISPLAY — prefer the device-code flow when the gesture comes over the wire"
created = 1791865750
updated = 1791865750
priority = 3
root_commit = "4dca48efee9e480f122f613931435d280a6ddedf"
tags = ["auth"]
+++
Observed 2026-10-12 on a seat box: a conversation stopped with 'token refresh failed for provider openai-chatgpt, and the ambient credential at ~/.codex/auth.json is absent or expired'. The window's /login (lernie login <ws> openai-chatgpt) runs the provider sign-in on the engine; the run's tail reads 'Error: no DISPLAY environment variable specified' because the yog user service carries no DISPLAY, so the browser-based authorize flow cannot start and the operator cannot recover from the seat. brazen's openai-chatgpt row declares a device flow (device = { url, style = codex }, 'the headless sign-in'). A sign-in asked for over the wire should use it, and print the URL and code into the login-tail the seat already paints; the browser flow is the right default only for bz run by hand on a box with a display.