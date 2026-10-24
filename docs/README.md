# Meeple documentation

Meeple is an ambient, procedurally generated isometric voxel world that plays
itself like a screensaver. These pages describe what it does and how it works.

| Page | What's in it |
| --- | --- |
| [Overview](overview.md) | Architecture, the frame loop, determinism, source layout |
| [Features](features.md) | A complete catalogue of everything the world contains |
| [Usage](usage.md) | Controls, URL options, quality tiers, what the browser remembers |
| [World generation](world-generation.md) | The seed-derived planner: terrain, biomes, rivers, towns, roads, railways, canals, sites |
| [Chunks and meshing](chunks-and-meshing.md) | Turning the plan into voxels, then into vertex data; streaming |
| [Simulation](simulation.md) | Clock, weather, villagers, goings-on, fauna, trains and carts, stations, travellers, skies, wonders, particles |
| [Autopilot](autopilot.md) | The camera director, stories, postcards, the camera itself |
| [Rendering](rendering.md) | The WebGL2 renderer, passes, shaders, adaptive quality |
| [Audio](audio.md) | The WebAudio synthesiser and the Rust-to-JS sound protocol |
| [Build, test and deploy](build-and-deploy.md) | Toolchain, Makefile, tests, the web shell, hot-swapping updates |

If you are new to the code, read the [overview](overview.md) first, then the page
for whichever subsystem you are changing.

> [!NOTE]
> The code is the source of truth. Tunable numbers (chances, radii, hours) are
> named constants near the top of the file that uses them; the docs name the
> file to look in rather than copying every value.
