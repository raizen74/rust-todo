Cargo.toml in the root of the project can handle multiple Cargo projects defined in the `[workspace]` section. The `-p` flag chooses the specific Cargo project for the command, such as `cargo run -p to-do-core`

We can compile the core nanoservice into another program or server by just
targeting the core **workspace**.