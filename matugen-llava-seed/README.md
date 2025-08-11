# matugen-llava-seed

`matugen-llava-seed` is a small Rust utility that bridges [LLaVA](https://github.com/haotian-liu/LLaVA) running via [Ollama](https://ollama.ai/) with [Matugen](https://github.com/Matugen/Matugen).  It asks a local LLaVA model for an accent color based on the current wallpaper, then invokes Matugen to generate themes and reloads common Wayland applications.

The project provides a zero‑fork shim — Matugen and your templates remain untouched.  The tool only decides on the seed color and triggers Matugen and reload hooks.

## Building

```bash
cd matugen-llava-seed
cargo build --release
install -Dm755 target/release/matugen-llava-seed ~/.local/bin/
```

Ensure `~/.local/bin` is on your `PATH`.

## Configuration

The tool looks for its config at `~/.config/matugen-llava-seed/config.toml`.  A sample config is shipped as [`default-config.toml`](default-config.toml).  Copy it to the config location to customise:

```bash
mkdir -p ~/.config/matugen-llava-seed
cp default-config.toml ~/.config/matugen-llava-seed/config.toml
```

Key sections:

- **[llava]** – endpoint, model name and generation options.
- **[matugen]** – how to call Matugen for color or image modes and any extra args (e.g. template sets).
- **[behavior]** – default light/dark mode, cache location and logging level.
- **[reload]** – enable/disable reload hooks for Hyprland, Waybar, Mako, Kitty and Fish.

## Usage

```
matugen-llava-seed <SUBCOMMAND> [FLAGS]
```

### analyze
Ask LLaVA for a seed color and print it:
```bash
matugen-llava-seed analyze --image ~/Pictures/wall.jpg
```

### apply
Analyze and immediately run Matugen.  Reload hooks fire only if Matugen succeeds:
```bash
matugen-llava-seed apply --image ~/Pictures/wall.jpg
```

Flags:
- `--mode auto|dark|light` – override mode (auto by default).
- `--force` – bypass cached seed color.
- `--dry-run` – show planned actions without changing anything.

### watch
Watch a directory for new wallpapers and apply themes automatically:
```bash
matugen-llava-seed watch --dir ~/Pictures/wallpapers --pattern '*.jpg,*.png'
```

### validate
Check that required executables (`matugen`, `kitty`, `waybar`, `makoctl`, `hyprctl`, `fish`) are available.

## Systemd units

Unit files are provided in [`systemd/`](systemd).  Install them under `~/.config/systemd/user`:

```bash
mkdir -p ~/.config/systemd/user
cp systemd/matugen-llava-seed.service ~/.config/systemd/user/
cp systemd/matugen-llava-seed.path ~/.config/systemd/user/
cp systemd/matugen-llava-seed.timer ~/.config/systemd/user/

# watch wallpaper directory
systemctl --user enable --now matugen-llava-seed.path

# optional daily refresh
systemctl --user enable --now matugen-llava-seed.timer
```

The service assumes the binary lives at `~/.local/bin/matugen-llava-seed`.

## Example Matugen templates

If you do not yet have templates, [`example-matugen-config.toml`](example-matugen-config.toml) shows how to wire Kitty and Waybar outputs.  Refer to the Matugen documentation for details.

## License

MIT

