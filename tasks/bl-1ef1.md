+++
title = "auto-reload the running yog instance after bl-install-main's CICD reinstall lands main"
created = 1785287445
updated = 1785287445
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
scripts/bl-install-main rebuilds+installs yog on every main merge but never touches the running instance. Add a Makefile 'reload' target (pkill -x yog, relaunch detached only if it was actually running — mirrors make ux's kill+relaunch idiom but non-blocking and conditional) and call it from bl-install-main's __build after a successful install. Update README's CICD section and make-target table.