# Seeds

Places in the world that show off what [the feature list](features.md) promises.
Every link opens a seed with the camera set down (`x` and `z`) beside the
thing in question; the [URL options](usage.md#url-options) say what the other
parameters do. Every link has `auto=0`, which starts with the autopilot
off so the camera stays where it landed; press `Space` to let it wander.

The `localhost` links expect `trunk serve --release` on port 8080 (see
[Usage](usage.md#running)); the production links open <https://meeple.jb3.dev>.

## One seed for nearly everything

**Seed 32** has 44 of the 45 things the finder looks for within 1.5 km of
the origin (the ski resort is the one it lacks), and 34 of them within 600
cells: a coastal village with a harbour, lighthouse, canal, weir and bridge close
together, and a city, tundra, desert and mountains all a short walk away.
Start at the village and let the autopilot wander:

| Seed | Feature | localhost | Production |
| --- | --- | --- | --- |
| 32 | Start here: harbour, lighthouse, canal, windmill, viaduct, ferry and bridge close by | [localhost](http://localhost:8080/?seed=32&x=-110&z=68&yaw=0.785&zoom=60&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-110&z=68&yaw=0.785&zoom=60&auto=0) |

Where a feature was more than 600 cells from seed 32's origin (the jungle, the
Northern and Tropical styles, the ski resort, chalk figure, monastery, hill
fort, mine and travellers' camp, which seed 32 either lacks or places too far
away), the row uses a different seed that has one close to its origin.

## Everything else

### Biomes

| Seed | Feature | localhost | Production |
| --- | --- | --- | --- |
| 32 | Plains | [localhost](http://localhost:8080/?seed=32&x=52&z=4&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=52&z=4&yaw=0.785&zoom=35&auto=0) |
| 32 | Forest | [localhost](http://localhost:8080/?seed=32&x=100&z=4&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=100&z=4&yaw=0.785&zoom=35&auto=0) |
| 32 | Desert (dunes and cacti) | [localhost](http://localhost:8080/?seed=32&x=-44&z=-236&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-44&z=-236&yaw=0.785&zoom=35&auto=0) |
| 32 | Tundra (frozen moors, snow) | [localhost](http://localhost:8080/?seed=32&x=4&z=100&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=4&z=100&yaw=0.785&zoom=35&auto=0) |
| 32 | Mountains (snow-capped) | [localhost](http://localhost:8080/?seed=32&x=-332&z=-92&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-332&z=-92&yaw=0.785&zoom=35&auto=0) |
| 91 | Jungle (kapoks, palms) | [localhost](http://localhost:8080/?seed=91&x=-44&z=-92&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=91&x=-44&z=-92&yaw=0.785&zoom=35&auto=0) |

### Towns and building styles

| Seed | Feature | localhost | Production |
| --- | --- | --- | --- |
| 32 | Village | [localhost](http://localhost:8080/?seed=32&x=-110&z=68&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-110&z=68&yaw=0.785&zoom=35&auto=0) |
| 32 | City (larger, with market stalls) | [localhost](http://localhost:8080/?seed=32&x=96&z=111&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=96&z=111&yaw=0.785&zoom=35&auto=0) |
| 32 | Village green (fairs, cricket, skating) | [localhost](http://localhost:8080/?seed=32&x=-76&z=269&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-76&z=269&yaw=0.785&zoom=35&auto=0) |
| 32 | Lowland style (timber and plaster) | [localhost](http://localhost:8080/?seed=32&x=-262&z=286&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-262&z=286&yaw=0.785&zoom=35&auto=0) |
| 32 | Highland style (stone and slate) | [localhost](http://localhost:8080/?seed=32&x=-475&z=123&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-475&z=123&yaw=0.785&zoom=35&auto=0) |
| 32 | Coastal style (whitewash, blue roofs) | [localhost](http://localhost:8080/?seed=32&x=-110&z=68&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-110&z=68&yaw=0.785&zoom=35&auto=0) |
| 42 | Northern style (log cabins) | [localhost](http://localhost:8080/?seed=42&x=-119&z=-88&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=42&x=-119&z=-88&yaw=0.785&zoom=35&auto=0) |
| 32 | Mediterranean style (terracotta roofs) | [localhost](http://localhost:8080/?seed=32&x=289&z=-303&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=289&z=-303&yaw=0.785&zoom=35&auto=0) |
| 119 | Tropical style (plank houses, green roofs) | [localhost](http://localhost:8080/?seed=119&x=-267&z=69&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=119&x=-267&z=69&yaw=0.785&zoom=35&auto=0) |

### Transport

| Seed | Feature | localhost | Production |
| --- | --- | --- | --- |
| 32 | Road bridge over a river | [localhost](http://localhost:8080/?seed=32&x=-160&z=71&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-160&z=71&yaw=0.785&zoom=35&auto=0) |
| 32 | Chain ferry over wide water | [localhost](http://localhost:8080/?seed=32&x=-106&z=134&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-106&z=134&yaw=0.785&zoom=35&auto=0) |
| 32 | Railway town with a station | [localhost](http://localhost:8080/?seed=32&x=-110&z=68&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-110&z=68&yaw=0.785&zoom=35&auto=0) |
| 32 | Railway viaduct over low ground | [localhost](http://localhost:8080/?seed=32&x=-34&z=41&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-34&z=41&yaw=0.785&zoom=35&auto=0) |
| 32 | Railway viaduct over water | [localhost](http://localhost:8080/?seed=32&x=0&z=43&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=0&z=43&yaw=0.785&zoom=35&auto=0) |
| 32 | Railway tunnel through a hill | [localhost](http://localhost:8080/?seed=32&x=-340&z=-192&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-340&z=-192&yaw=0.785&zoom=35&auto=0) |
| 32 | Railways crossing (signals, set points) | [localhost](http://localhost:8080/?seed=32&x=105&z=64&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=105&z=64&yaw=0.785&zoom=35&auto=0) |
| 32 | Junction curve between lines | [localhost](http://localhost:8080/?seed=32&x=140&z=-46&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=140&z=-46&yaw=0.785&zoom=35&auto=0) |
| 32 | Passing loop | [localhost](http://localhost:8080/?seed=32&x=-239&z=-649&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-239&z=-649&yaw=0.785&zoom=35&auto=0) |
| 32 | Canal with towpaths and narrowboats | [localhost](http://localhost:8080/?seed=32&x=52&z=12&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=52&z=12&yaw=0.785&zoom=35&auto=0) |

### Countryside sites

| Seed | Feature | localhost | Production |
| --- | --- | --- | --- |
| 32 | Windmill | [localhost](http://localhost:8080/?seed=32&x=-65&z=84&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-65&z=84&yaw=0.785&zoom=35&auto=0) |
| 32 | Watermill | [localhost](http://localhost:8080/?seed=32&x=55&z=-151&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=55&z=-151&yaw=0.785&zoom=35&auto=0) |
| 32 | Manor | [localhost](http://localhost:8080/?seed=32&x=-107&z=-258&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-107&z=-258&yaw=0.785&zoom=35&auto=0) |
| 32 | Quarry | [localhost](http://localhost:8080/?seed=32&x=-323&z=-278&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-323&z=-278&yaw=0.785&zoom=35&auto=0) |
| 32 | Ruin | [localhost](http://localhost:8080/?seed=32&x=-447&z=259&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-447&z=259&yaw=0.785&zoom=35&auto=0) |
| 32 | Lighthouse | [localhost](http://localhost:8080/?seed=32&x=-103&z=120&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-103&z=120&yaw=0.785&zoom=35&auto=0) |
| 32 | Harbour with jetty and fishing boats | [localhost](http://localhost:8080/?seed=32&x=-131&z=100&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-131&z=100&yaw=0.785&zoom=35&auto=0) |
| 20 | Hill mine | [localhost](http://localhost:8080/?seed=20&x=129&z=92&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=20&x=129&z=92&yaw=0.785&zoom=35&auto=0) |
| 32 | Standing stones | [localhost](http://localhost:8080/?seed=32&x=-617&z=-88&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-617&z=-88&yaw=0.785&zoom=35&auto=0) |
| 109 | Chalk-hill figure | [localhost](http://localhost:8080/?seed=109&x=-69&z=-236&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=109&x=-69&z=-236&yaw=0.785&zoom=35&auto=0) |
| 32 | Geyser | [localhost](http://localhost:8080/?seed=32&x=-437&z=-121&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-437&z=-121&yaw=0.785&zoom=35&auto=0) |
| 32 | Desert oasis | [localhost](http://localhost:8080/?seed=32&x=290&z=-436&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=290&z=-436&yaw=0.785&zoom=35&auto=0) |
| 100 | Ski resort | [localhost](http://localhost:8080/?seed=100&x=318&z=-98&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=100&x=318&z=-98&yaw=0.785&zoom=35&auto=0) |
| 74 | Monastery | [localhost](http://localhost:8080/?seed=74&x=-61&z=49&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=74&x=-61&z=49&yaw=0.785&zoom=35&auto=0) |
| 32 | Wayside shrine | [localhost](http://localhost:8080/?seed=32&x=-247&z=81&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-247&z=81&yaw=0.785&zoom=35&auto=0) |
| 34 | Hill fort | [localhost](http://localhost:8080/?seed=34&x=-236&z=96&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=34&x=-236&z=96&yaw=0.785&zoom=35&auto=0) |
| 120 | Travellers' camp | [localhost](http://localhost:8080/?seed=120&x=-55&z=-43&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=120&x=-55&z=-43&yaw=0.785&zoom=35&auto=0) |
| 32 | Orchard | [localhost](http://localhost:8080/?seed=32&x=-148&z=500&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-148&z=500&yaw=0.785&zoom=35&auto=0) |
| 32 | Apiary | [localhost](http://localhost:8080/?seed=32&x=-60&z=101&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-60&z=101&yaw=0.785&zoom=35&auto=0) |
| 32 | Weir across a river | [localhost](http://localhost:8080/?seed=32&x=-90&z=-48&yaw=0.785&zoom=35&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-90&z=-48&yaw=0.785&zoom=35&auto=0) |

### Times of year, and options

| Seed | Feature | localhost | Production |
| --- | --- | --- | --- |
| 32 | Winter snow in a city | [localhost](http://localhost:8080/?seed=32&x=96&z=111&season=winter&hour=11&yaw=0.785&zoom=30&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=96&z=111&season=winter&hour=11&yaw=0.785&zoom=30&auto=0) |
| 32 | Autumn colours, and salmon leaping the weir | [localhost](http://localhost:8080/?seed=32&x=-90&z=-48&season=autumn&hour=11&yaw=0.785&zoom=25&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-90&z=-48&season=autumn&hour=11&yaw=0.785&zoom=25&auto=0) |
| 32 | Spring blossom in a village | [localhost](http://localhost:8080/?seed=32&x=-110&z=68&season=spring&hour=11&yaw=0.785&zoom=30&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-110&z=68&season=spring&hour=11&yaw=0.785&zoom=30&auto=0) |
| 32 | A city at night, with glowing windows | [localhost](http://localhost:8080/?seed=32&x=96&z=111&hour=22&yaw=0.785&zoom=30&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=96&z=111&hour=22&yaw=0.785&zoom=30&auto=0) |
| 32 | Northern lights over the tundra | [localhost](http://localhost:8080/?seed=32&x=4&z=100&hour=23&aurora=1&yaw=0.785&zoom=45&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=4&z=100&hour=23&aurora=1&yaw=0.785&zoom=45&auto=0) |
| 32 | Looking inside a house (zoom right in) | [localhost](http://localhost:8080/?seed=32&x=-110&z=68&hour=11&yaw=0.785&zoom=8&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=-110&z=68&hour=11&yaw=0.785&zoom=8&auto=0) |
| 32 | Chunky pixels | [localhost](http://localhost:8080/?seed=32&x=96&z=111&pixel=5&hour=17&yaw=0.785&zoom=30&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=96&z=111&pixel=5&hour=17&yaw=0.785&zoom=30&auto=0) |
| 32 | The easter egg: Bella lifts weights in every square | [localhost](http://localhost:8080/?seed=32&x=96&z=111&bella&hour=11&yaw=0.785&zoom=14&auto=0) | [meeple.jb3.dev](https://meeple.jb3.dev/?seed=32&x=96&z=111&bella&hour=11&yaw=0.785&zoom=14&auto=0) |

## How the places were found

`find_feature_spots` in `src/world/chunk/seed_hunt.rs` scans seeds 1–120 out to
1100 cells from the origin and prints the instance of each feature nearest the
origin (`HUNT|seed|feature|x|z|distance`), and the busiest 700-cell square of each
seed (`CLUSTER|seed|distinct features|x|z`). Biomes are sampled on a 48-cell grid;
towns, sites, weirs, ferries and railway crossings come from the planner;
bridges, viaducts and tunnels come from scanning the columns beside roads and
rails (only for the 20 seeds with the most variety). It isn't run with the normal
tests:

```bash
cargo test --release --target "$(rustc -vV | sed -n 's/^host: //p')" find_feature_spots -- --ignored --nocapture
```

`HUNT_LO` and `HUNT_HI` set the range of seeds. The finder reports where
something is generated, not what the camera sees; the links frame each spot at
a zoom that shows the feature, but a feature in an unusual spot (say, a ski
resort on a steep slope) may need a nudge with `WASD`.

Features that depend on the time of day, the season or the weather (fires,
weddings, the travelling fair, a total eclipse) happen in whichever town the
autopilot is watching, so they have no fixed place. Use `T` to speed time up and
`Space` to let the autopilot go find them, or the last table for the fixed
looks.
