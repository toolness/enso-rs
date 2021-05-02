This is a very experimental rewrite of [Enso][] in Rust.

It currently only works on Windows.

## Quick start

```
cargo run
```

## Installation

To install Enso to run at startup, first install it:

```
cargo install --path .
```

Then, find the Enso executable in Explorer (it will likely be in `~/.cargo/bin`).

Open the Startup folder by pressing <kbd>Windows</kbd> + <kbd>R</kbd> and
typing `shell:startup`.

Then create a shortcut to the Enso executable in the Startup folder.

Once you reboot your system, Enso should start.

[Enso]: https://github.com/toolness/community-enso
