# Features

A catalogue of everything the world does. Each section points at the code and
the page with the details.

## The world

**Terrain and climate** ([details](world-generation.md))
- Rolling continents, hills and detail noise, mountains from ridged noise, stepped
  terraces, meandering river valleys cut below sea level, lakes and sea.
- Five biomes: plains, forest, desert (dunes, cacti, palms at oases), tundra (frozen
  moors, wolves, aurora), mountain (snow-capped, ski resorts, geysers).
- Cold climates have log-cabin towns and snow earlier; peaks are snowy from late
  autumn to early spring.

**Towns and cities**
- One potential town per 192-unit region; villages (radius 24–35) and cities (38–51).
- Street grid with a central plaza, fountain, benches and (in cities) permanent
  market stalls; lots 8×8 voxels.
- Building kinds: house, apartment block, tavern/inn, bakery, smithy (with anvil and
  forge), shop with awning, church, farm (with fields and crops), garden, school,
  library.
- Four regional styles: Lowland (timber and plaster), Highland (stone and slate),
  Coastal (whitewash and blue roofs), Northern (log cabins).
- Names for towns and people from `names.rs`.
- Houses are built over the first seventy days (plot, scaffolding, frame, done), a
  few are abandoned later and fall to ruin with ivy.
- A village green outside most towns, levelled flat, for fairs, cricket and skating.

**Transport**
- Roads between neighbouring towns in long S-bends; bridges over rivers; **chain
  ferries** over water ten cells or wider.
- **Railways** at five angles across the land; viaducts over low ground and water,
  tunnels through hills, signals and junction curves where lines cross, passing
  loops, a station with platform and canopy in rail towns.
- **Canals** dug through low land between stretches of open water, with towpaths.
- Telegraph poles beside the railways from the second age.

**Countryside sites** ([details](world-generation.md))
Windmills (by villages), watermills (on rivers, with a water race), manors,
quarries, ruins (mostly haunted), lighthouses and harbours with jetties, hill mines
(winding wheel, trucks, lamplit tunnel), standing stones (with a midsummer sunrise
stone), chalk-hill figures, geysers, desert oases with camel caravans, ski resorts
with a lift and chalets, monasteries (with a bell), wayside shrines, hill forts,
travellers' camps, orchards and apiaries, weirs across rivers, anglers on banks.

## Time, weather and sky

- A day/night cycle (default 20 real minutes a day; `?day=`); four seasons of
  three days each; sunrise and sunset move with the season; an eight-day moon.
- Weather: clear, cloudy, rain, storm, fog, snow, with season-dependent odds,
  wind, lightning with delayed thunder, snow that settles and melts, wet ground and
  steam when the sun returns.
- **Rivers** swell after long heavy rain and run low in dry summers; dry spells make
  thatch burn more easily; storms can strike a house with lightning.
- **Harvest**: luck of the year, drought and flood decide how heaped the market
  stalls are and how loaded the carts.
- **Seasonal looks**: spring blossom, summer green, autumn colours and falling leaves,
  bare winter trees, snow cover, ice, crops growing, ripening and being cut.
- Sky: gradient with sun and moon, stars and clouds with moving cloud shadows,
  fog, mist at dawn, **rainbows** that stand in the world, **northern lights** in cold
  country, shooting stars, a rare **total eclipse** at noon every month or so.
- Three **ages**: horse and lantern, steam and telegraph, motor and electric light;
  a world starts in one and moves on every 36 days (cars replace carts, telegraph
  poles appear).
- Airships, biplanes and hot-air balloons (still mornings and evenings).

## Villagers ([details](simulation.md))

- Twelve roles with their own schedules: farmer, baker, innkeeper, blacksmith,
  shopkeeper, vicar, teacher, librarian, postie, villager, child, elder.
- Daily routines: work, lunch, errands, visits to friends, the inn, the plaza, benches;
  school on weekday mornings; weather-aware (nobody loiters in the rain, umbrellas).
- Chatting with emotes (talk, heart, note, laugh), babble in their own pitch.
- **Families**: shared surnames; babies born (the same ones whenever a town loads),
  pram, children, grown-up children who may take over a parent's trade; elders.
  Click someone to see their kin.
- **Markets** every third morning (stalls round the plaza, baskets), the baker's bread
  run to the inn, the postie's rounds.
- **Evening gatherings**: maypole (spring), midsummer jig, harvest festival,
  winter bonfire with lanterns; dancers, singers and a musician; children's games
  (tag, ball, skipping).
- **Weddings** in cities: guests and couple in church, bells and petals, a dance;
  the couple then share a home.
- Pets: dogs, cats; farm chickens.
- **Looking in on houses**: zoom right in and the nearest house with someone at home
  is laid open like a dolls' house, with a furnished room (`I` turns it off).
- Villager card (click), village newsletter (`J`), chronicle (`C`).

## Goings-on ([details](simulation.md))

Fires and the brigade's bucket chain; christenings; dogs chased across the square;
the doctor's rounds; closing time at the inn; the travelling fair (carousel, ferris
wheel, strongman, stalls, fireworks); cricket; skating; snowmen and snowball
fights; the farming year's chores; the lamplighter and the fiddler at the inn door.

## Animals ([details](simulation.md))

Sheep and cows on pasture, ducks, swans and herons on water, deer, foxes and rabbits
at dawn and dusk, owls at night, wolves in the cold north, migrating deer herds in
spring and autumn, salmon leaping weirs in autumn, birds crossing the sky and
great migrating skeins, gulls following fishing boats, sailing and fishing boats,
narrowboats on the canals.

## Machines and moving parts

Steam trains with locomotives, coaches, freight and log wagons; semaphore signals
and set points; horse carts and (later) cars; windmill sails turning with the wind;
waterwheels; lighthouse beams; ski lifts with skiers; mine winding gear and ore
trucks; geyser eruptions; ferry barges; airships, biplanes, balloons; telegraph
messages that run as sparks along the wires; weir water with foam and mist.

## Camera and interaction

- A full **autopilot** ([details](autopilot.md)) that chooses what to watch by time and
  weather, follows people, trains, boats and travellers, goes to festivals, weddings
  and fires (and tells the town it is coming so it happens), follows multi-day story
  arcs, makes occasional long journeys, and holds still for captioned postcards.
- Manual pan, zoom, rotate; click a villager or a train to follow.
- See-through buildings and trees so the people being watched are never hidden.
- Photograph mode with a tilt-shift look and PNG saving; shareable links to an exact
  spot and moment; a map; time-lapse; world of the day and a hand-picked gallery.
- Touch support: pan, pinch-zoom, tap, on-screen menu.
- Returning to a world resumes its clock.

## Rendering ([details](rendering.md))

WebGL2, instanced boxes for everything that moves, baked ambient occlusion, shadow
maps with a close-up cascade, SSAO, light shafts, depth of field, TAA, HDR bloom,
water reflections and caustics, colour grading, seasonal shading in the shader, GPU
weather, night lighting with up to 16 nearest point lights and glowing windows,
adaptive quality over six tiers, chunky-pixel mode.

## Sound ([details](audio.md))

A fully synthesised soundscape (no samples): weather, birds and insects, water,
crowds, children, the inn, bells and the church clock, trains (chuffing, clickety-clack,
horn, doppler), animals, machinery, a calliope, fireworks, ghosts; generative music
that is different in every world and place, with a fiddle and choir for gatherings
and their own music for fires, weddings, christenings and closing time.

## Easter eggs and tools

- `?bella`: a man called Bella lifts weights in front of a mirror in every town
  square.
- `?prof`, `?bench`, `?fxoff`, `?aurora`, `?pitch` for development and screenshots.
- `web/sounds.html`: audition every sound.
