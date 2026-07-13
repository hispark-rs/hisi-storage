# hisi-storage

Chip-neutral, `no_std`, read-first storage contracts for the hispark-rs
ecosystem. The initial stable surface is bounded memory-mapped reading through
`embedded-storage`; flash erase/write remains experimental until the platform
XIP and power-loss invariants are proven.
