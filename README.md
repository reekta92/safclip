# SafClip

SafClip is a terminal user interface (TUI) video clipping tool written in Rust. It offers keyboard and mouse-driven controls to create and merge video clips losslessly.

## Key Features

* **Lossless Cuts**: Uses FFmpeg stream copying (`-c copy`) to cut and merge video files instantly without re-encoding.
* **Keyframe Snapping**: Queries keyframes via `ffprobe` and snaps segment boundaries to them, preventing frozen or black frames at clip start.
* **MPRIS Integration**: Syncs playback status, seek position, and track duration with active Linux media players (like MPV or VLC) over D-Bus.
* **Zoomable Timeline**: A TUI timeline that can be zoomed and panned for millisecond-level precision.
* **Mouse Scrubbing**: Click to seek, drag to scrub, scroll wheel to zoom (anchored at cursor), and right-drag to pan the timeline.
* **Session Restoration**: Saves segments to `<source_filename>.safclip.json` automatically, with modification verification on reload.

## Prerequisites

* `ffmpeg` and `ffprobe` in your system `PATH`.
* Linux OS with D-Bus for MPRIS player synchronization.

## Usage

1. Start an MPRIS-compatible media player:
   ```bash
   mpv --mpris /path/to/video.mp4
   ```
2. Run SafClip:
   ```bash
   safclip-controller /path/to/video.mp4
   ```

## Keybindings

### Navigation
* `Space`: Play / Pause
* `Left` / `Right`: Seek ±1s (Shift: ±5s, Alt: ±10s)
* `Home` / `End`: Jump to start / end
* `K`: Snap to nearest keyframe

### Timeline
* `+` / `-`: Zoom in / out
* `Alt` + `h` / `l`: Pan left / right

### Mouse (Timeline)
* **Left Click**: Seek
* **Left Drag**: Scrub (pauses during scrub)
* **Scroll Wheel**: Zoom (anchored at cursor)
* **Right Drag**: Pan

### Segments & Export
* `a` / `d` (or `Enter`): Set IN point / Set OUT point (adds segment)
* `Up` / `Down`: Select previous / next segment
* `Delete` / `x`: Delete selected segment
* `u` / `Ctrl` + `r`: Undo / Redo
* `e` / `E` (or `Shift` + `e`): Export separate clips / Export merged clip

### General
* `Tab`: Switch active MPRIS player
* `?`: Toggle help popup
* `Esc`: Cancel or close popups
* `q`: Quit

## Session File Format

Sessions are saved as `<source_filename>.safclip.json`:

```json
{
  "version": 1,
  "source_path": "/path/to/video.mp4",
  "source_modified": 1717372800,
  "segments": [
    {
      "id": "c3b8a1c8-c68e-4a69-873f-c300c76db36d",
      "start_seconds": 12.5,
      "end_seconds": 45.2,
      "label": null
    }
  ]
}
```
