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
- **Cell:** The smallest spatial unit in the game, represented by a three-dimensional cube. Each cell is exactly one of: ooze, a single resource type, or an organism body cell.
- **Resource:** A typed material stored in a resource cell as a quantity. Resources do not move; they are consumed or generated in place.
- **Organism:** A contiguous group of body cells that share one body type, backed by an organism instance that holds state separate from its body cells. Organisms may be controlled by the player or act independently.
- **Experience (XP):** A measure of the player's activity and progress within the current world. Interacting with the world and growing controlled organisms grants XP, which determines the player's XP level.
- **Knowledge:** The collection of abilities available to the player in the current world. Knowledge is acquired through discovery and by reaching new XP levels, making it the primary progression system. Unlocking all Knowledge is required to complete the game. Both XP and Knowledge are stored with the world state and do not carry over to a new game.
- **Time:** The simulation advances continuously through discrete ticks. Organisms age over time and eventually die.
- **Visibility:** The player can identify only cells within the visibility range of controlled organisms. That range is measured from the outer extent of each organism and varies by type and abilities. Cells outside this range, and cells that later fall outside it, are displayed as **Unknown**.

## Cells

Nothing in the world changes position. Organisms, resources, and individual cells remain fixed in place. Instead, cells change type or state in response to nearby cells. Growth, consumption, production, decay, organism splitting, and organism merging all occur through cell transformations.

There are no moving projectiles. Effects that might otherwise be represented by projectiles must instead propagate through local cell interactions or fields of positive or negative influence.

Each cell is exactly one of three kinds: **ooze**, a **resource** cell holding one resource type, or an **organism body** cell. A cell cannot combine types—for example, a resource cell cannot also be part of an organism. Ooze is effectively inert, empty space.

**Resource cells** store the resource type and amount on the cell. **Body cells** store the body type and an association to the organism instance they belong to. **Organism-level state**—such as age, health, and control—is tracked on the organism instance, not on individual body cells.

Organism identity is determined by connectivity: all contiguous body cells of the same body type form one organism and reference a single organism instance. If two body cells of the same type touch, they belong to the same organism. Disconnected groups of the same body type are separate organisms with separate instances. Transformations can connect or separate body-cell groups, allowing organisms to merge or split.

Local adjacency for resource interaction uses **face neighbors** only—the six cells that share a face with a given cell. For player actions, a cell is **in contact** with a player-controlled organism when it is face-adjacent to at least one of that organism's body cells.

## Organisms

Organisms are contiguous groups of body cells that share a single body type and form irregular three-dimensional shapes. Each organism type has a distinct set of properties, behaviors, and abilities.

An organism contains only one body type. Body cells that touch an organism of a different body type belong to a separate organism, even if they are adjacent.

Organisms reveal nearby cells, allowing the player to discover more of the world. Each organism type defines how far beyond its outer extent it reveals surrounding cells.

Some organisms are player-controlled, while others operate independently. Independent organisms may be harmful, neutral, or beneficial. Environmental influences may cause an organism to transition between controlled and independent states. When a controlled organism becomes independent, the player loses control of it.

Independent organisms outside the active simulation remain frozen until they activate. An inactive independent organism becomes active when an active player-controlled organism comes within its **activation range**. That range is defined by the inactive organism's attributes and may extend beyond the player-controlled organism's visibility range.

The player can seed a new organism when the required resources and knowledge are available. Required resources must be in contact with a player-controlled organism. Seeding requires sufficient resources; if the player has none accessible this way, they cannot establish new organisms. The current interaction concept uses a right-click tool to transform an ooze cell into the selected organism type.

Nearby organisms interact through local rules. An interaction may benefit, harm, or have no effect on either organism, depending on the organisms' types and properties.

### Organism state

An organism's state is tracked as a whole, separate from its body cells. Body cells hold only spatial membership—their body type and a reference to the organism instance they represent.

### Split and merge

When an organism splits, each resulting organism begins with the same age, health, and control as the parent. As additional organism properties are defined, split and merge behavior must be specified per property in the organism catalog. Most properties will simply duplicate to each new organism; others may be divided proportionally, possibly based on the volume (number of body cells) of each new body.

## Resources

Resources occur naturally in the world and may also be produced by organisms. Depending on the consuming or touching organism, a resource may be beneficial, harmful, or inert. Harmful resources are a primary threat to the player's organisms.

### Cell content and quantities

A resource cell holds exactly one resource type and tracks an **amount** of that resource. Amounts are initially represented as integers; some resource types or consumption patterns may later require real values. Resource quantities are stored on the resource cell itself, not on organisms.

When a resource cell is fully consumed, it immediately becomes an ooze cell.

### Adjacency and access

Resources do not move between cells. Consumption and generation change the amount in place, or convert a cell's type.

For an organism to consume from or generate into a resource cell, that resource cell must be **face-adjacent** to at least one of the organism's body cells (sharing one of the six faces).

A resource cell touched by only one face of an organism's body can be **shared**—multiple organisms may consume from or interact with it. For an organism to have **exclusive** access, all six faces of the resource cell must be adjacent to that organism's body cells.

The player has access to any resource cell **in contact** with a player-controlled organism (face-adjacent to at least one of its body cells), whether or not that organism can process the resource. An organism that cannot process a resource can still surround it and block other organisms from reaching it.

### Consumption and generation

Resource consumption, generation, transformation, and combination are performed only by organisms.

An organism consumes or generates a resource only if it has an ability that processes that resource type. Generation into a new resource cell converts an ooze cell, or an organism body cell other than the organism's last remaining body cell, into a resource cell of the generated type. An organism generates a resource cell only when no other resource cells of that type are available to it—that is, none are in contact with the organism.

Transformations that combine resources into more advanced types are also performed by organisms, drawing on Vita over time at a rate governed by the organism's Vita use efficiency.

### Player reallocation

The player may transfer an amount of a resource from one resource cell to another, provided both cells are **in contact** with a player-controlled organism—that is, face-adjacent to at least one of its body cells. The player cannot create new resource amounts, convert cells to resource types, or transform or combine resources; those actions are reserved for organisms. The player cannot manipulate body cells of independent or hostile organisms. Player transfers are immediate; distance and time are not factors.

Resource quantities are initially displayed only through the cell inspection window.

### Organism changes and resource effects

Changes to an organism—split, merge, change of control, death, and so on—do not alter the resource cells it touches. The quantity in each resource cell remains on that cell. After a split, one part of the organism may lose access to a resource if its body no longer face-adjacent to that cell.

Resource effects are determined exclusively by the organism consuming or touching the resource, not by universal properties of the resource type alone.

## Vita

Vita is an ambient energy concept, not a quantifiable resource. The world assumes an infinite supply is always available everywhere—in active and inactive regions alike. Consumption in one area is immediately replenished; the simulation does not model Vita flow, localized Vita quantities, or depletion.

Each organism has a **Vita use efficiency** property that determines how effectively it draws on ambient Vita and therefore the rate of its Vita-dependent processes. Other organisms and resources, within a defined range, can raise or lower this efficiency for nearby organisms. These effects are interpreted as locally increasing Vita concentration or suppressing Vita availability, without simulating actual Vita movement or storage.

### Environmental systems (future)

Temperature, ooze density, and similar environmental properties are parked as future concepts. Whether and how they enter the simulation will be decided if and when they are brought into the game.

## Resource Types

The resource catalog will define each available resource type and its identity in the world.

## Organism Types

The organism catalog will define each available organism type, including its properties, abilities, behavior, visibility range, activation range, Vita use efficiency, resource requirements, resource production and transformation rules, effects when consuming or touching resources, split and merge rules per property, and interactions with other organisms.

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

- What happens to an organism's cells when it dies: immediate conversion to ooze, gradual decay, or transformation into resources?
- How do effects propagate without movement, and what limits their range, speed, and duration?

### Control and Player Actions

- What does “player-controlled” mean when organisms do not move? Which behaviors can the player configure or trigger?
- Under what conditions can an organism become independent or return to player control?
- Does seeding create a single body cell, a complete starter organism, or a growth process?
- Is right-click permanently reserved for applying the selected tool, and how does the player choose organism types and inspect cells?
- Are actions instantaneous, tick-based, or powered over time by Vita?

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
