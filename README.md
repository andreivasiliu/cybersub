# CyberSub

[![dependency status](https://deps.rs/repo/github/andreivasiliu/cybersub/status.svg)](https://deps.rs/repo/github/andreivasiliu/cybersub)

This is made from the template repo for [egui](https://github.com/emilk/egui/) (found [here](https://github.com/emilk/egui_template)), with eframe later replaced for [macroquad](https://github.com/not-fl3/macroquad) for more efficient rendering.

Currently this is just a prototype for handling water on a 2D grid, with pressure and inertia for each cell, in the context of a submarine with pumps, doors, and destructible walls.

It builds as both a native desktop application and a WASM-powered web page.

To see it in action, check: https://andreivasiliu.github.io/cybersub/

Previous prototypes:
* egui and eframe (using widgets to draw cells, makes phones spontaneously combust): https://andreivasiliu.github.io/cybersub/proto/1

The project's name is a working title for a simpler pixel-art clone of [Barotrauma](https://barotraumagame.com/) that I had in my head, which will likely never come to fruition, but is fun to think about and build towards anyway.
