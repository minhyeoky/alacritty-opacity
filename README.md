# alacritty-opacity

A simple command-line tool to adjust [Alacritty](https://github.com/alacritty/alacritty)'s opacity configuration.

## Usage

Install alacritty-opacity.

```bash
cargo install alacritty-opacity
```

Increase or decrease opacity by value.

```bash
# the value must be between 0.0 ~ 1.0
alacritty-opacity increase 0.1
alacritty-opacity decrease 0.1
```

Currently i'm using with key bindings for Tmux.
(Press `)` or `(` after prefix key to adjust opacity)

```tmux
bind-key -r ) run-shell -b "alacritty-opacity increase 0.1"
bind-key -r ( run-shell -b "alacritty-opacity decrease 0.1"
```
