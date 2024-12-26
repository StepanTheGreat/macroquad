# TODO

1. Remove any global state functionality from macroquad
2. Expose the removed state functionality as separate modular components
3. Make different crate sub-modules, with relevant exports:
    - Graphics (everything graphics related)
    - Text (font loading, rendering)
    - Window (window management, methods)
    - Filesystem (some filesystem functions, mostly just 2 blocking implementations)
    - Input
    - Audio (everything sound related)
    - UI