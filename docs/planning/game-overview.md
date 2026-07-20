# Game Overview

**Status:** concept draft  
**Source:** revised from [game-overview-sketch.md](game-overview-sketch.md)

## Overview

The game takes place in a finite, three-dimensional biological environment composed of cellular organisms, raw resources, inert ooze, and an ambient energy called **Vita**. Vita permeates the world and powers biological processes.

The player is an entity external to this environment. Rather than controlling an avatar, the player directly manipulates any visible part of the world. The player begins with limited knowledge and a small set of abilities, then unlocks new capabilities by exploring and interacting with the environment.

At the start of a game, the player can see and use the resources within a small spherical region around a randomly chosen starting point near the center of the world. The player may navigate freely throughout the entire world, but cells outside the current visibility range of controlled organisms appear as unknown gray cells and reveal no information.

## Objectives and Failure Conditions

### Primary objective

The player's primary objective is to expand their controlled organisms across the world until they reach the edges of the world boundary. Along the way, the player discovers and unlocks **Knowledge**—the abilities required to grow, adapt, and survive. Completing the game requires both reaching the world boundary and unlocking all Knowledge available in that world.

### Threats and pressure

The player's organisms are continuously threatened by harmful resources and independent organisms. Survival depends on managing growth, resource production, and interactions with the environment while pushing outward into increasingly dangerous territory.

### Failure conditions

The player loses the game when they have no controlled organisms remaining. A player loses control of an organism when that organism becomes independent or dies.

Resource depletion creates additional pressure. If the player runs out of resources, they can no longer seed new organisms. Without the ability to establish new controlled organisms, the player may eventually lose their last remaining organism and fail the game.

### Progression and persistence

Progression applies only within a single generated world. XP, Knowledge, and other player advancement are stored as part of the world state. Starting a new game creates a new world with no carried-over progression from previous games.

The game is single-player only. There is one player per world, with no multiplayer or shared-world support.

## Core Concepts

- **Vita:** An ambient energy concept that permeates the world. Vita is not a quantifiable resource—the world assumes an infinite supply is always available everywhere. Organisms draw on Vita to power biological processes at a rate governed by their **Vita use efficiency**.
- **Cell:** The smallest spatial unit in the game, represented by a three-dimensional cube. A cell may contain ooze, a resource, or part of an organism.
- **Resource:** Raw or processed material that organisms can consume, store, produce, or transform.
- **Organism:** A contiguous group of body cells that share one body type. Organisms may be controlled by the player or act independently.
- **Experience (XP):** A measure of the player's activity and progress within the current world. Interacting with the world and growing controlled organisms grants XP, which determines the player's XP level.
- **Knowledge:** The collection of abilities available to the player in the current world. Knowledge is acquired through discovery and by reaching new XP levels, making it the primary progression system. Unlocking all Knowledge is required to complete the game. Both XP and Knowledge are stored with the world state and do not carry over to a new game.
- **Time:** The simulation advances continuously through discrete ticks. Organisms age over time and eventually die.
- **Visibility:** The player can identify only cells within the visibility range of controlled organisms. That range is measured from the outer extent of each organism and varies by type and abilities. Cells outside this range, and cells that later fall outside it, are displayed as **Unknown**.

## Cells

Nothing in the world changes position. Organisms, resources, and individual cells remain fixed in place. Instead, cells change type or state in response to nearby cells. Growth, consumption, production, decay, organism splitting, and organism merging all occur through cell transformations.

There are no moving projectiles. Effects that might otherwise be represented by projectiles must instead propagate through local cell interactions or fields of positive or negative influence.

Each cell has a type representing ooze, a resource, or part of an organism. Ooze is effectively inert, empty space. Organism identity is determined by connectivity: all contiguous body cells of the same body type form one organism. If two body cells of the same type touch, they belong to the same organism. Disconnected groups of the same body type are separate organisms. Transformations can connect or separate these groups, allowing organisms to merge or split.

## Organisms

Organisms are contiguous groups of body cells that share a single body type and form irregular three-dimensional shapes. Each organism type has a distinct set of properties, behaviors, and abilities.

An organism contains only one body type. Body cells that touch an organism of a different body type belong to a separate organism, even if they are adjacent.

Organisms reveal nearby cells, allowing the player to discover more of the world. Each organism type defines how far beyond its outer extent it reveals surrounding cells.

Some organisms are player-controlled, while others operate independently. Independent organisms may be harmful, neutral, or beneficial. Environmental influences may cause an organism to transition between controlled and independent states. When a controlled organism becomes independent, the player loses control of it.

Independent organisms outside the active simulation remain frozen until they activate. An inactive independent organism becomes active when an active player-controlled organism comes within its **activation range**. That range is defined by the inactive organism's attributes and may extend beyond the player-controlled organism's visibility range.

The player can seed a new organism when the required resources and knowledge are available. Seeding requires sufficient resources; if the player has none, they cannot establish new organisms. The current interaction concept uses a right-click tool to transform an ooze cell into the selected organism type.

Nearby organisms interact through local rules. An interaction may benefit, harm, or have no effect on either organism, depending on the organisms' types and properties.

## Resources

Resources occur naturally in the world and may also be produced by organisms. Depending on the consuming organism, a resource may be beneficial, harmful, or inert. Harmful resources are a primary threat to the player's organisms.

Each resource has properties that determine how it can be consumed and transformed. Transformations generally combine one or more resources into a more advanced resource and draw on Vita over time at a rate governed by the organism's Vita use efficiency.

A resource cell contains a finite quantity of one resource. When that quantity is exhausted, the cell becomes ooze. Some organisms naturally produce or store resources within their body cells.

The player can use only resources that are currently visible. Most raw resources cannot be used directly and must first be processed by an organism. Processed resources are stored in one or more cells within that organism's body.

## Vita

Vita is an ambient energy concept, not a quantifiable resource. The world assumes an infinite supply is always available everywhere—in active and inactive regions alike. Consumption in one area is immediately replenished; the simulation does not model Vita flow, localized Vita quantities, or depletion.

Each organism has a **Vita use efficiency** property that determines how effectively it draws on ambient Vita and therefore the rate of its Vita-dependent processes. Other organisms and resources, within a defined range, can raise or lower this efficiency for nearby organisms. These effects are interpreted as locally increasing Vita concentration or suppressing Vita availability, without simulating actual Vita movement or storage.

### Environmental systems (future)

Temperature, ooze density, and similar environmental properties are parked as future concepts. Whether and how they enter the simulation will be decided if and when they are brought into the game.

## Resource Types

The resource catalog will define each available resource, including its properties, production and transformation rules, storage behavior, and effects on different organisms. A resource may have organism-specific effects as well as universal effects.

## Organism Types

The organism catalog will define each available organism type, including its properties, abilities, behavior, visibility range, activation range, Vita use efficiency, resource requirements, and interactions with other organisms.

## World Generation

The complete world is generated when a new game begins, although only a small region is initially visible. Each new game creates a fresh world with its own progression state.

### World scale

The world is a cube. Its extent along each axis is defined in code (currently `WORLD_SIZE` in `boxes_sim`), not as a design-time parameter in this document.

### Starting point

The player's starting point is chosen at random within a fixed radius of the world center. World generation begins from that point and expands outward, placing resources and organisms so that a viable progression path exists from the start.

### Navigation

The player can navigate freely throughout the entire world, including through unknown cells. Navigation is not limited to visible or previously discovered regions.

### Visibility

A cell is visible when it lies within a certain distance of the outer extent of a controlled organism. That distance varies by organism type and abilities. Cells beyond this range render as **Unknown** and provide no information to the player.

Player-controlled organisms are always visible, along with all cells within their visibility range. A controlled organism cannot exist in an unknown or inactive off-screen state.

Visibility is not permanent. If a cell later falls outside the visibility range of all controlled organisms—for example, because organisms die, shrink, or lose visibility range—the cell becomes **Unknown** again. The player does not retain a remembered view of its last known state.

### Disconnected visible regions

Visible regions may become separated when controlled organisms die or visibility is lost between them. The player can still interact with each disconnected visible region, because navigation across unknown cells is unrestricted.

### World boundaries

World boundaries are impassable voids. Cells cannot exist or grow beyond the cube boundary.

Visibility expands as the player's organisms grow and spread. If controlled organisms die, previously connected visible regions may become separated by unknown space.

### Simulation outside visible areas

Inactive independent organisms are excluded from time progression and remain frozen in their current state. They do not age, consume resources, produce resources, fight, or die while inactive. This keeps them out of sync with the active world, but it is likely necessary for game performance. It also prevents high-level independent organisms from advancing toward the player before the player is ready.

Invisibility and inactivity are not the same. An organism may be hidden from the player while still active. An inactive independent organism becomes active when an active player-controlled organism enters its **activation range**, which is defined by the inactive organism's attributes. Because this range may extend beyond the player-controlled organism's visibility range, an independent organism can become active—and begin simulating—before the player can see it. This creates a band at the edge of the visible world where organisms may be active but still unknown to the player.

When an inactive organism becomes active, it resumes from the state it was in when it became inactive. If it has never been active during the game, it resumes from the state it had when generated. Elapsed time while inactive is not simulated in batch.

Activation and visibility are governed by separate, deterministic rules: activation depends on distance to active player-controlled organisms, and visibility depends on controlled-organism visibility range. Because unknown cells reveal no information regardless of what is simulating behind them, off-screen activation does not indirectly inform the player about hidden areas. Determinism and information hiding are implementation concerns for the simulation layer, not open design questions at this level.

Resource and organism distribution is part of the progression model. More advanced or dangerous content is placed farther from the starting point as generation expands outward from the player's initial location.

## Open Questions

### Player Goals and Progression

- Exactly which actions grant XP, how much XP do they grant, and can repeated low-risk actions be exploited?
- What is unlocked through XP levels versus discovery? Can knowledge be lost, hidden, or acquired in multiple ways?
- How are advanced resources and organisms introduced so that discovery feels intentional rather than arbitrary?

### Cells, Organisms, and Identity

- Does every cell have both a type and mutable state, and which data belongs to the cell versus the organism as a whole?
- When an organism splits, how are stored resources, age, health, control, and other organism-level properties divided?
- What happens to an organism's cells when it dies: immediate conversion to ooze, gradual decay, or transformation into resources?
- Which local neighborhood rules are used—six face neighbors, all 26 surrounding cells, or a type-specific range?
- How do effects propagate without movement, and what limits their range, speed, and duration?

### Control and Player Actions

- What does “player-controlled” mean when organisms do not move? Which behaviors can the player configure or trigger?
- Under what conditions can an organism become independent or return to player control?
- Can hostile or independent organisms be directly manipulated once visible, or only influenced indirectly?
- When seeding an organism, where are the required resources drawn from? Must they be adjacent, connected through an organism network, or merely visible anywhere?
- Does seeding create a single body cell, a complete starter organism, or a growth process?
- Is right-click permanently reserved for applying the selected tool, and how does the player choose organism types and inspect cells?
- Are actions instantaneous, tick-based, or powered over time by Vita?

### Resources and Transformation

- Can a cell contain only one resource type, and can it contain both a resource and an organism body cell?
- How are resource quantities represented, transferred, and displayed?
- If cells never move, how do resources travel between cells or between organisms?
- What determines whether a raw resource is usable, and which organism must process it?
- Are transformed resources globally available to the player, or must they remain physically stored and connected to where they are used?
- What happens to stored resources when their organism splits, changes control, or dies?
- Are resource effects intrinsic to the resource, specific to the consuming organism, or resolved by an explicit combination of both?

### Biomes and World Generation

- Should the world contain explicit biomes with authored properties, or should regional differences emerge entirely from cell composition?
- If biomes exist, which rules do they affect: resource distribution, Vita, organism behavior, visibility, temperature, or transformation rates?
- How is difficulty measured when placing content farther from the starting point?
- What guarantees that a generated world contains the resources and organism chains required for progression?
- Are independent organisms generated in a fixed initial state, procedurally grown, or simulated for a period before play begins?
- How much of world generation must be deterministic from a seed?

### Content Definition and Balance

- Which organism and resource types are required for an initial playable version?
- What common property model allows organism interactions without defining every pair individually?
- How are positive, negative, and neutral interactions communicated to the player?
- How long should growth, processing, aging, and death take in real time and in simulation ticks?
- What prevents stable systems from becoming permanently self-sustaining or, conversely, collapsing without recoverable resources?
- Which mechanics are essential to the core game, and which—such as environmental fields and control transitions—should be deferred until later?
