# How character animation via sprite sheets works, in general

Two parts to the mental model: (1) what a sprite sheet actually *is* and how
"frames" are addressed within it, and (2) how playback over time gets
orchestrated. Both are engine-agnostic — Unity, Godot, Phaser, Bevy, and a
hand-rolled engine all solve this with roughly the same shape. Bevy specifics
are noted in parentheses only as a concrete illustration.

## 1. A sprite sheet is one image + a slicing scheme

A spritesheet isn't secretly a bunch of separate images — it's a single
texture with multiple sub-images packed into it. "Loading" it really means
two separate things:

- Load the texture once (one GPU upload, one file read).
- Separately define a set of rectangles *into* that texture, one per frame.
  This "frame index → pixel rect" mapping is the atlas layout/metadata.

Two common ways that mapping gets built:

- **Uniform grid** — fixed frame size, fixed columns/rows; frame N's rect is
  computed arithmetically. This is what you get from a clean N-frame strip
  like `Idle.png`. (Bevy: `TextureAtlasLayout::from_grid`.)
- **Packed/irregular atlas** — frames of varying sizes tightly packed to save
  space, with an accompanying data file (JSON/XML) mapping names or indices
  to arbitrary rects. Produced by tools like TexturePacker, Aseprite, or
  Unity's Sprite Atlas — necessary once frames aren't uniform (mixed
  animations, mixed bounding boxes).

Either way, the runtime ends up with the same shape of data: an ordered list
of rects into one texture.

Why bother with one big image instead of many small ones? Draw-call/texture-
binding overhead — switching the GPU's currently-bound texture between draws
is one of the more expensive things a renderer does per frame, so packing
everything a character (or a whole animation set, or even unrelated sprites
like UI icons) needs into one texture lets the renderer batch draws instead
of constantly rebinding. That's a rendering-pipeline concern, not something
specific to any one engine.

## 2. Per-entity: "which frame right now" + a clock advancing it

The atlas layout above is shared, static data — loaded once, reused by every
entity using that sheet. Each individual character on screen additionally
needs its own tiny bit of state:

- **current frame index** (or pointer/name) — which rect it's showing right now
- **a clock** — something accumulating elapsed time and deciding when to
  step to the next index

This is universally a timer-driven loop: accumulate `delta_time`, and once
enough has passed for one frame at your target playback rate (pixel-art
games often deliberately run animation at a chunky low rate — 6–12 "frames
per second" of animation — independent of the actual render framerate),
advance the index by one, wrapping back to the clip's start once you hit the
end. (Bevy: an `AnimationTimer` component ticked in a system, indexing into
`Sprite.texture_atlas.index`.) Every engine's built-in animation player is a
fancier version of exactly this loop.

## 3. Organizing multiple animations ("clips")

A character doesn't have one strip of frames — it has several: idle, walk,
attack, hurt, etc. The universal concept is a **named clip = a contiguous
range of frame indices** (whether within one shared sheet or its own
separate texture is just an implementation choice). "Playing an animation"
reduces to: point the entity at clip X's texture/atlas, and constrain the
looping index to clip X's range.

Bigger engines formalize this as an **animation state machine** (Unity's
Animator/AnimatorController, Godot's AnimationTree, Aseprite's "tags" that
most engines import directly) — a small graph of clip nodes with transitions
triggered by gameplay state (is the character moving? attacking? just took
damage?). Under the hood it's the same primitive: swap which clip's index
range is active, driven by whatever gameplay signal indicates the
character's current action. Even a minimal hand-rolled version — e.g.
checking "is there a move-in-progress component right now" to decide
idle-vs-walk — is a small instance of that same pattern.

## 4. Direction is usually just another axis, or handled by flipping

For a character that can face multiple directions, two standard approaches
show up across nearly every top-down or platformer engine:

- **Full directional art** — extend the grid so each row is a direction and
  each column is an animation frame; direction and frame become two
  independent axes multiplied together to get an index. Costs the most art
  (a full cycle per direction).
- **Mirroring** — draw only left-or-right (plus up/down separately) and
  horizontally flip the sprite in code for the opposite side. Every engine's
  sprite renderer supports a cheap horizontal-flip flag for exactly this
  reason — it roughly halves the art budget at the cost of not being able to
  have asymmetric character art.

## 5. Locomotion speed can be tied to real movement, not just a flat clock

One refinement worth knowing conceptually: rather than a walk cycle ticking
on a fixed timer, more polished games drive the animation's playback rate
off actual movement speed/distance, so faster movement visibly speeds up the
footstep animation instead of the character sliding with mismatched feet.
This is an optional layer over the same "timer advances an index" primitive
— the "how much time counts as one frame" value becomes dynamic instead of
constant.
