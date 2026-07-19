# Game Overview

**Status:** concept draft  
**Source:** revised from [game-overview-sketch.md](game-overview-sketch.md)

## Overview

The game takes place in a finite, three-dimensional biological environment composed of cellular organisms, raw resources, inert ooze, and an ambient energy called **Vita**. Vita permeates the world and powers biological processes.

The player is an entity external to this environment. Rather than controlling an avatar, the player directly manipulates any visible part of the world. The player begins with limited knowledge and a small set of abilities, then unlocks new capabilities by exploring and interacting with the environment.

At the start of a game, the player can see and use the resources within a small spherical region around a starting point. The player may navigate the view throughout the pre-generated world, but cells outside the current visibility network appear as unknown gray cells and reveal no information.

## Core Concepts

- **Vita:** Ambient energy that flows throughout the world. Organisms consume Vita to power growth and other biological processes.
- **Cell:** The smallest spatial unit in the game, represented by a three-dimensional cube. A cell may contain ooze, a resource, or part of an organism.
- **Resource:** Raw or processed material that organisms can consume, store, produce, or transform.
- **Organism:** A contiguous, multicellular structure that occupies an irregular three-dimensional shape. Organisms may be controlled by the player or act independently.
- **Experience (XP):** A measure of the player's activity and progress. Interacting with the world and growing controlled organisms grants XP, which determines the player's XP level.
- **Knowledge:** The collection of abilities available to the player. Knowledge is acquired through discovery and by reaching new XP levels, making it the primary progression system.
- **Time:** The simulation advances continuously through discrete ticks. Organisms age over time and eventually die.
- **Visibility:** The player can identify only cells within the visibility range of controlled organisms. All other cells are displayed as **Unknown**.

## Cells

Nothing in the world changes position. Organisms, resources, and individual cells remain fixed in place. Instead, cells change type or state in response to nearby cells. Growth, consumption, production, decay, organism splitting, and organism merging all occur through cell transformations.

There are no moving projectiles. Effects that might otherwise be represented by projectiles must instead propagate through local cell interactions, bursts of Vita, or fields of positive or negative influence.

Each cell has a type representing ooze, a resource, or part of an organism. Ooze is effectively inert, empty space. Contiguous body cells belonging to the same organism form one organism; disconnected groups form separate organisms. Transformations can connect or separate these groups, allowing organisms to merge or split.

## Organisms

Organisms are contiguous groups of body cells arranged in irregular three-dimensional shapes. Each organism type has a distinct set of properties, behaviors, and abilities.

Organisms reveal nearby cells, allowing the player to discover more of the world. Their visibility ranges may vary by type.

Some organisms are player-controlled, while others operate independently. Independent organisms may be harmful, neutral, or beneficial. Environmental influences may cause an organism to transition between controlled and independent states.

The player can seed a new organism when the required resources and knowledge are available. The current interaction concept uses a right-click tool to transform an ooze cell into the selected organism type.

Nearby organisms interact through local rules. An interaction may benefit, harm, or have no effect on either organism, depending on the organisms' types and properties.

## Resources

Resources occur naturally in the world and may also be produced by organisms. Depending on the consuming organism, a resource may be beneficial, harmful, or inert.

Each resource has properties that determine how it can be consumed and transformed. Transformations generally combine one or more resources into a more advanced resource and consume Vita over time.

A resource cell contains a finite quantity of one resource. When that quantity is exhausted, the cell becomes ooze. Some organisms naturally produce or store resources within their body cells.

The player can use only resources that are currently visible. Most raw resources cannot be used directly and must first be processed by an organism. Processed resources are stored in one or more cells within that organism's body.

## Vita

Vita is ambient energy distributed throughout the world. Organisms draw on it directly to power processes over time.

Future organism or resource types may alter local Vita availability by concentrating or suppressing it. Specialized organisms may also store Vita and release it in larger bursts—for example, to charge a defensive mechanism.

## Resource Types

The resource catalog will define each available resource, including its properties, production and transformation rules, storage behavior, and effects on different organisms. A resource may have organism-specific effects as well as universal effects.

## Organism Types

The organism catalog will define each available organism type, including its properties, abilities, behavior, visibility range, resource requirements, and interactions with other organisms.

## World Generation

The complete world is generated when a new game begins, although only a small region is initially visible. The world is finite and will likely be cubic for implementation simplicity.

Visibility expands as the player's organisms grow and spread. If controlled organisms die, previously connected visible regions may become separated by unknown space.

Organisms outside visible areas are intended to remain inactive until they are discovered or come within an awareness range of the player's visible territory.

Resource and organism distribution is part of the progression model. More advanced or dangerous content should generally appear farther from the starting point. This progression model may require placing the player near the center of the world rather than at a fully random location.

## Open Questions

### Player Goals and Progression

- What is the player's primary objective, and what constitutes victory or completion?
- What threats, failure states, or lose conditions create pressure on the player?
- Exactly which actions grant XP, how much XP do they grant, and can repeated low-risk actions be exploited?
- What is unlocked through XP levels versus discovery? Can knowledge be lost, hidden, or acquired in multiple ways?
- Does progression apply only within one generated world, or does anything persist between games?
- How are advanced resources and organisms introduced so that discovery feels intentional rather than arbitrary?

### World Scale, Navigation, and Visibility

- What are the world's dimensions, and is its shape definitely a cube?
- Is the starting point random, fixed near the center, or selected under constraints that guarantee a viable progression path?
- Can the camera navigate freely through unknown space, or only through visible and previously discovered regions?
- Is visibility based on distance from every controlled organism, a connected network, line of sight, or some combination of these?
- Does a cell become unknown again when visibility is lost, or does the player retain a remembered view of its last known state?
- If visible regions become disconnected, can the player interact with every region equally, or does interaction require a connection to a primary organism or network?
- Are world boundaries inert walls, impassable voids, wraparound edges, or something else?

### Simulation Outside Visible Areas

- Time is described as continuous and organisms age, but organisms in non-visible areas are described as inactive. Do hidden organisms age, consume resources, produce resources, fight, and die?
- If hidden areas are paused, how are they reconciled when discovered? Do they resume from their original generated state or simulate elapsed time in a batch?
- Does “inactive” apply only to independent organisms, or also to player-controlled organisms that are temporarily outside visibility?
- What causes an undiscovered organism to become aware of the player's territory, and can that occur before the player can see it?
- How can off-screen activation remain deterministic and avoid revealing information indirectly?

### Cells, Organisms, and Identity

- Does every cell have both a type and mutable state, and which data belongs to the cell versus the organism as a whole?
- How is organism identity represented? If two organisms of the same body type touch, do they automatically merge?
- Can one organism contain multiple body-cell types, or must all contiguous cells share one organism type?
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

### Vita and Environmental Systems

- Is Vita initially uniform and unlimited, or does consuming it temporarily or permanently deplete local availability?
- If Vita flows, what direction and rate govern that flow when nothing in the world moves?
- How do Vita concentration and suppression interact with the claim that Vita is distributed evenly throughout the world?
- Can Vita cross unknown or inactive regions, and is its simulation paused there?
- Is stored Vita represented as cell state, a resource, or an organism-level value?
- Are temperature, ooze density, or other environmental properties global fields, cell states, or emergent effects of nearby cells?
- The possibility of “ooze currents” appears inconsistent with the no-movement rule. Would currents move material, propagate influence without movement, or be excluded?

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
- Which mechanics are essential to the core game, and which—such as Vita storage, environmental fields, and control transitions—should be deferred until later?
