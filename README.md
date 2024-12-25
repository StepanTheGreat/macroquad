# macroquad_abstractions

This is a fork of macroquad, whose sole purpose is to turn macroquad into a `library`.
Macroquad has a lot of useful abstractions for miniquad, however, macroquad abstracts a lot from the user
into its own private state. In some cases, one might want to create their **own** framework, while using
useful abstractions from macroquad.

This is exactly the point of this crate! All macroquad abstractions are exported, and some are optionally
locked behind crate features, so you can avoid reinventing the wheel, and still have the freedom
to customize your own state!

## Additional changes
- Minimal code changes
- Remove async abstractions (this crate is supposed to be used with miniquad)
- Transform async operations into blocking ones (or you can of course simply use miniquad's approach)

## Licensing
Like the original macroquad crate, this crate is licensed under MIT and Apache-2.0, under your choice.
