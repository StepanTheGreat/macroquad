# macroquad_abstractions

This is a fork of macroquad, whose sole purpose is to turn macroquad into a `library`.
Macroquad has a lot of useful abstractions for miniquad, however, macroquad abstracts a lot from the user
into its own private state. In some cases, one might want to create their **own** framework, while using
useful abstractions from macroquad.

This is exactly the point of this crate! All macroquad abstractions are exported, and some are optionally
locked behind crate features, so you can avoid reinventing the wheel and still have the freedom
to customize your own state!

## Before using
1. This crate tries to be minimal, so it doesn't introduce most of the global state that's present in macroquad,
   including the auto resource cleaning. That means that `TextureId`, `SoundId`, `GlPipeline`, `Material` and so on are now your responsibility to manage, properly clean and remove.
2. Some new abstractions (like `Texture`) were made solely to simplify some macroquad operations. They aren't
   required, and exist solely for convenience.
3. A lot of rendering operations are implemented exclusively for `Renderer<Vertex>`. I thought it would be a great
   feature overall for people that like the batcher but would like to use their own Vertex type, so I added this 
   generic parameter. If you create a renderer with a different `Vertex` type - you won't be able to use the 
   default macroquad operations (as they have no idea how to construct anything).
4. Some things simply don't make sense (like text rendering, atlas and so on) as they were designed to be used
   with the global state. For now they're hardly usable, but you can use them as a starting point for something of
   your own.
5. The sound API is almost entirely removed, so you'll be working with `quad-snd` types directly instead

I might miss something, but the global point I'll try to make is that this crate is best used as a fork, so you can
tweak some types, add something of your own, or, add it on top instead. If you don't have the time however to
reinvent anything - this crate might be a good start.

## Licensing
Like the original macroquad crate, this crate is licensed under MIT and Apache-2.0, under your choice.
