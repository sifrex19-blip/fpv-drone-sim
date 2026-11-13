# Controller images

Thumbnails for the Controller Database. Lookup order in `src/ui.rs`:

1. `<slug>.png` — a real product photo for that exact controller
2. `_kind_<Kind>.png` — generated fallback illustration (Radio, Gamepad, Joystick, …)

## File naming

File names are slugs of the controller name from `src/controllers.rs`:
lowercase, every run of non-alphanumeric characters becomes a single `_`.

    "Radiomaster TX16S MkII"     -> radiomaster_tx16s_mkii.png
    "Joy-Con (L)"                 -> joy_con_l.png
    "Thrustmaster T.Flight HOTAS X" -> thrustmaster_t_flight_hotas_x.png

## Sources

- `tools/fetch-controller-images.ps1 -Repo <repo-root>` fetches freely licensed
  photos (Wikimedia Commons first, Openverse CC0/PD/BY/BY-SA as fallback),
  normalizes them to 240x180 letterboxed on the theme background, and writes
  attribution to `CREDITS.md`.
- `tools/gen-icons.ps1` generates the `_kind_*.png` fallbacks.

Entries without a fetched photo simply keep the kind illustration.
