# ewmh-status-listener

Simple EWMH-compatible window-manager status listener.
It listens to all state changes and outputs one line json to stdout with the current state for each state change (e.g. `_NET_ACTIVE_WINDOW` has changed because of the user focused another window)

The output looks something like this (formatted for better readability, this is outputted in one line):
```json
{
  "desktops": [
    {
      "name": "1",
      "id": 0,
      "windows": [
        {
          "resource_id": 48234541,
          "name": "Eww expressions - eww documentation — Mozilla Firefox",
          "desktop_id": 0,
          "states": ["MaximizedVert", "MaximizedHorz"]
        }
      ]
    },
    {
      "name": "2",
      "id": 1,
      "windows": []
    },
    {
      "name": "3",
      "id": 2,
      "windows": [
        {
          "resource_id": 52428811,
          "name": "cargo run ~/d/p/r/ewmh-status-listener",
          "desktop_id": 2,
          "states": []
        }
      ]
    }
  ],
  "current_desktop_id": 2,
  "active_window": {
    "resource_id": 52428811,
    "name": "cargo run ~/d/p/r/ewmh-status-listener",
    "desktop_id": 2,
    "states": []
  }
}
```

The following window states are supported:
* `Modal`
* `Sticky`
* `MaximizedVert`
* `MaximizedHorz`
* `Shaded`
* `SkipTaskbar`
* `SkipPager`
* `Hidden`
* `Fullscreen`
* `Above`
* `Below`
* `DemandsAttention`

It's currently mostly intended for [eww](https://github.com/elkowar/eww) as `deflisten` target for a workspaces widget.
But it may be for use for different applications as well.

## Installation/Running

There is a Nix flake (e.g. can be run with flakes enabled nix via `nix run github:Philipp-M/ewmh-status-listener`), otherwise it can e.g. be installed with cargo (needs the dependency `libxcb`):

```bash
cargo install --git https://github.com/Philipp-M/ewmh-status-listener.git
ewmh-status-listener
```

## Usage with eww

It can be used e.g. like this in a workspaces widget in eww:

```yuck
(deflisten ewmh-state "ewmh-status-listener")
(defwidget workspaces-ewmh []
  (eventbox
    :onscroll "echo '(${ewmh-state.current_desktop_id}{}+${arraylength(ewmh-state.desktops)})%${arraylength(ewmh-state.desktops)}' | sed -e \"s/up/+1/g\" -e \"s/down/-1/g\" | bc | xargs wmctrl -s"
    (box
      :class "module workspaces"
      (for ws in {ewmh-state.desktops}
        (button
          :onclick "wmctrl -s ${ws.id}"
          :visible {arraylength(ws.windows) > 0 || ws.id == ewmh-state.current_desktop_id}
          :class "ws ${ws.id == ewmh-state.current_desktop_id ? "active" : arraylength(ws.windows) > 0 ? "inactive" : "empty"}"
          :tooltip {jq(ws, '[.windows[] | .name] | join(", ")')} ; TODO formatting could be improved
          "${ws.name}")))))
```
