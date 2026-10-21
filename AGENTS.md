# ClearSleeper — rules for agents working in this tree

- **Do not build.** Do not run `cargo build`, `cargo check`, `cargo test`, `cargo clippy` or
  `cargo bench` here, and do not set `CARGO_TARGET_DIR` to reach a different target directory.
  The coordinator owns the build: it restores the target directory, runs one check and one
  test pass over the whole tree, and reports the result. An agent's job ends with the edits
  and the findings, not with a compile.
- `cargo fmt --all` is fine: it is one fast pass over the files that changed.
- If a build is needed to answer a question, say so in the report instead of running it. Do
  not work around a blocked or failing build (no symlinked target directories, no alternate
  target paths, no reduced checks to make a build pass).
- Every behaviour change needs a decomp citation (`tww/src/...:line`) next to it, and the
  decomp under `/home/ymsr/Documents/Games/repame/tww` is the only behaviour reference.
- Do not commit, push, stash, or reset; the coordinator commits the tree.
