# Restaurats Mod Manager (egui)

**Any and all references to Restaurats is attributed to:**

```yaml
DEVELOPER: toR Studio
PUBLISHER: Polden Publishing
```

A simple Rust desktop app (egui/eframe) to manage mods for Restaurats.

Features:

- Install BepInEx (auto: v5.4.23.4 via GitHub API; or custom URL/ZIP for BE IL2CPP)
- Apply Unity 6 workaround: sets `UnityLogListening = false` in `BepInEx.cfg`
- Drag-and-drop mod ZIPs to install (extracts to `BepInEx/plugins`)
- Uninstall mods cleanly via tracked file list
- Play button to launch `Restaurats.exe`

## Default game path

`C:\\Program Files (x86)\\Steam\\steamapps\\common\\Restaurats`

You can change it via the Browse button.

## Installing BepInEx

- Click "Install v5.4.23.4 (auto)" to download and extract the latest v5 LTS Windows x64 zip.
- For Unity 6 IL2CPP, paste a BE IL2CPP zip URL in the custom URL field and click Install, or choose "Install from ZIP..." after downloading manually.
- Click "Apply UnityLogListening=false" to set the recommended logging flag.

Note: Writing to Program Files may require Administrator privileges. If install fails with PermissionDenied errors, run the app as Administrator.

## Build & Run

Requires Rust toolchain (stable).

```powershell
cd c:\Users\micha\development\memory
cargo run --release
```

## Mod Zips

- Thunderstore-style zips with `BepInEx/plugins/...` are installed directly into game root.
- If no structure is found, top-level `.dll` files are placed into `BepInEx/plugins`.

The manager keeps an index at `BepInEx/mod-manager.index.json` for uninstall.

## Limitations

- Only `.zip` archives are supported.
- For Bleeding Edge IL2CPP builds, supply a valid zip URL or file.

## Developer Note

I hope you find this useful! This is built as a tool for my friends to install mods as bepinex was a bit complex.
