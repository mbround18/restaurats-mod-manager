Place your Restaurats logo image here as logo.png.

Embedding (bundling into the .exe):
- Put the file at assets/logo.png
- Build with the `embed-logo` feature enabled:
  cargo run --release --features embed-logo

Runtime load (not embedded):
- Optionally place assets/logo.png next to the executable to be loaded at runtime if not embedded.
