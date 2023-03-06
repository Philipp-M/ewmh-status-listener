# ewmh-status-listener

Simple EWMH-compatible window-manager status listener.
It listens to all state changes and outputs one line json to stdout with the current state for each state change (e.g. `_NET_ACTIVE_WINDOW`)

It's currently mostly intended for [eww](https://github.com/elkowar/eww) as `deflisten` target for a workspaces widget.
But it may be for use for different applications as well.

It contains a Nix flake, otherwise it can e.g. be installed with cargo (needs the dependency `libxcb`):

```bash
cargo install --path .
```
