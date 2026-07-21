+++
title = "S0 polish: default viewport too small on first launch"
created = 1784605269
updated = 1784605269
root_commit = "805ddf08f8a13f1d0c2b0bf7b07d4a1bc438706c"
+++
Live walk evidence (scratchpad yog-final.png): NativeOptions::default() at src/main.rs:90 yields a first-launch window where the S0 surface does not fit (roster 280 logical + center slivers; GNOME HiDPI). Fix: NativeOptions{ viewport: egui::ViewportBuilder::default().with_inner_size([1150.0, 760.0]).with_min_inner_size([700.0, 500.0]), ..Default::default() } — logical points, winit handles scale. main.rs is coverage-excluded; gate must stay green.