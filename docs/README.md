# Restaurats Mod Manager (egui)

**Any and all references to Restaurats is attributed to:**

```yaml
DEVELOPER: toR Studio
PUBLISHER: Polden Publishing
```

A simple Rust desktop app (egui/eframe) to manage mods for Restaurats.

## Download & Run

- Grab the latest Windows `.exe` from [GitHub Releases](https://github.com/mbround18/restaurats-mod-manager/releases)
- Place it anywhere (or inside the game folder) and run it; no install required.

Features:

- Install BepInEx (auto: bleeding-edge BE IL2CPP build by default; or custom URL/ZIP)
- Apply Unity 6 workaround: sets `UnityLogListening = false` in `BepInEx.cfg`
- Drag-and-drop mod ZIPs to install (extracts to `BepInEx/plugins`)
- Uninstall mods cleanly via tracked file list
- Play button to launch `Restaurats.exe`

## Default game path

`C:\\Program Files (x86)\\Steam\\steamapps\\common\\Restaurats`

You can change it via the Browse button.

## Installing BepInEx

- Click "Install Bleeding Edge (auto)" to download and extract the bundled IL2CPP Windows x64 zip.
- Or paste a custom BE IL2CPP zip URL and click Install, or choose "Install from ZIP..." after downloading manually.
- Click "Apply UnityLogListening=false" to set the recommended logging flag.

Note: Writing to Program Files may require Administrator privileges. If install fails with PermissionDenied errors, run the app as Administrator.

## Build (optional)

If you prefer to build yourself: `cargo build --release` (Rust stable required). The shipped `.exe` from releases needs no build.

## Mod Zips

- Thunderstore-style zips with `BepInEx/plugins/...` are installed directly into game root.
- If no structure is found, top-level `.dll` files are placed into `BepInEx/plugins`.

The manager keeps an index at `BepInEx/mod-manager.index.json` for uninstall.

## Limitations

- Only `.zip` archives are supported.
- For Bleeding Edge IL2CPP builds, supply a valid zip URL or file.

## Developer Note

I hope you find this useful! This is built as a tool for my friends to install mods as bepinex was a bit complex.
