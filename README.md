# smart-road

## Prerequisites

The application uses native SDL2 and SDL2_mixer libraries for graphics and music:

```bash
# Fedora
sudo dnf install SDL2-devel SDL2_mixer-devel

# Ubuntu/Debian
sudo apt install libsdl2-dev libsdl2-mixer-dev
```

Run it with `cargo run`. Press `M` or click the button in the top-right corner to mute or resume the music.
