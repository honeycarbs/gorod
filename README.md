# Gorod

A tile-based city-building simulation game written in Rust.

![Game Demo](doc/demo.png)

## Theme fulfillment

This game simulates a living city where population grows or shrinks based on housing availability, jobs, and citizen happiness. Every in-game day, the simulation recalculates population movement, worker productivity, income and expenses, and happiness levels. Buildings can be abandoned if conditions deteriorate. So yes, it is a **simulation that evolves over time**.

## Project Structure

Gorod is built using the Bevy game engine with the Entity Component System (ECS) architecture.

### Dependencies

- `Bevy` (of course)
- [`bevy_ecs_tilemap`](https://github.com/StarArawn/bevy_ecs_tilemap) - this is an amazing crate that was built by a very talented member of the community. After lookinh through many-many crates with tilemaps, I can conclude that this is the only one that fullfills ECS in full and it was perfect for this project. Of course it has a _major_ flaw, but I will talk later about that.

### Module Organization

The codebase is organized into the following modules:

**budget/** - Manages city finances including income calculation, upkeep costs, and transaction processing. Displays budget information in the UI.

**camera/** - Handles camera movement (WASD/arrow keys) and zoom controls (scroll wheel).

**city/** - Core simulation logic. Contains systems for population growth, happiness calculation, demand computation, and building abandonment. This is where the daily simulation tick runs.

**map/** - Tile placement and demolition. Manages the placeable area expansion, tile highlighting, building sprites, and road connectivity. Handles all user interaction with the map.

**spatial/** - Spatial hash map optimization for neighbor queries. Buildings are indexed by grid cell to speed up distance-based lookups (road accessibility, nearby residential count).

**time/** - In-game clock and time controls. Manages simulation speed multipliers and day/night progression.

### Key Implementation Details

The game uses Bevy's plugin system to organize functionality. Each module defines its own plugin that registers resources, events, and systems:

- `GameTimePlugin` - Time progression and speed controls
- `BudgetPlugin` - Financial tracking and UI
- `CameraControllerPlugin` - Camera input handling
- `TilePlacementPlugin` - Map interaction and building placement
- `SimulationPlugin` - Population and happiness simulation

The simulation runs on a daily tick. At normal speed (1x), one in-game day takes approximately 8.6 real seconds. Systems communicate through Bevy events like `BuildingPlaced`, `BuildingDemolished`, and `TransactionFailed`.

## What went wrong, eat went right and lessons learned

### The good

I was impressed with Bevy and what projects can be done in Bevy, and how easy it is to do ECS with Bevy. I tried to write my own toy ECS crate before I discovered Bevy, and it was very hard.

All assets were hand drawn by yours truly except for the font - I used [Silkscreen](https://fonts.google.com/specimen/Silkscreen).

ECS was a discovery for me and this project in general was my "soft launch" in gamedev. Solo gamedev is really fun, but you have to do a lot to be talented in tons of stuff to be the next ConcernedApe.

### The bad

First, I wanted to make an isometric city builder. But `bevy_ecs_tilemap` has a major flaw - it's Y-sorting is [broken](https://github.com/StarArawn/bevy_ecs_tilemap/discussions/491). I tried to fix it myself, but turns out I'm really not that smart.

With that, I had less time than I hoped and I did nit implement saving mechanics, even though it is really simple - just a matrix in a txt file, and lots of fun stuff that I planned. But if I don't stop now, I will burn out, and I really want to finish this game some day.

Overall, Rust is a really young language and it definitely shows with Bevy - it is impressive, but definitely small. For gamedev I would use Godot for sure.

### The AI

I asked Opus (very smart model btw!) to check if my game mechanics were balanced (they were not, you can see how many commits it took to make it at least decent), and now this game is sort of playable.

---

## In case you want to play this little game

### Goal

Build a thriving city by balancing housing, jobs, entertainment, and budget. Keep your citizens happy to encourage population growth.

### Controls

| Key | Action |
|-----|--------|
| R | Select Residential building |
| C | Select Commercial building |
| I | Select Industry building |
| O | Select Road |
| B | Select Decorative building |
| , / . | Cycle through building variants |
| Left Click | Place selected building |
| Shift + Left Click | Demolish building |
| Space | Pause/Resume simulation |
| 1 / 2 / 3 | Set simulation speed |
| WASD or Arrow Keys | Move camera |
| Scroll Wheel | Zoom in/out |
| ? | Toggle help overlay |

### Building Types

**Residential** - Houses for your citizens. Each building provides 10 housing capacity.

**Commercial** - Shops and businesses. Provides 5 jobs and 15 entertainment. Generates income from workers.

**Industry** - Factories. Provides 15 jobs and 3 entertainment (I dunno because labour is fun I guess?). Higher income per worker but more expensive upkeep.

**Roads** - Required for buildings to function. Buildings must be within 4 tiles of a road to contribute to city statistics. **Buildings need to be near roads to function.**

**Decorative** - Parks and decorations (well, only a single park I should say). Provides 20 entertainment but has high upkeep cost.

### Core Mechanics

**Population**: People move into your city based on available housing and job opportunities. High happiness (above 70%) enables immigration, allowing population to exceed housing capacity. Population adjusts toward its target by 35% each day.

**Happiness**: Ranges from 0% to 100%. Affected by housing shortage, job shortage, and entertainment shortage. When happiness drops below 70%, buildings may be abandoned every 3 days.

**Budget**: You start with $50,000. Income comes from worker taxes and business profits. Commercial and Industry buildings have daily upkeep costs. Running a negative balance for 3+ days decreases happiness.

**Expansion**: You start with a small 3x3 buildable area in the center. Placing any building expands the buildable area by 2 tiles in all directions.

