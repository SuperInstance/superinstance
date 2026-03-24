# 📸 Screenshots & GIFs

This folder contains visual assets for the SuperInstance README and documentation.

## Required Screenshots

| File | Description | Status |
|:-----|:------------|:-------|
| `dashboard.gif` | TUI + Web dashboard side-by-side view | 📝 Placeholder |
| `breeding.gif` | TensorRT-LLM breeding with geometric snapping | 📝 Placeholder |
| `memory.gif` | Memory Pasture RAG with CRDT in action | 📝 Placeholder |
| `install.gif` | Single-binary install process | 📝 Placeholder |

## How to Capture

Run the following on your Jetson Orin Nano:

```bash
# Terminal recording with asciinema
asciinema rec dashboard.cast

# Convert to GIF
agg dashboard.cast dashboard.gif

# Or use ttygif
ttygif dashboard.cast
```

## Recommended Recording

1. **dashboard.gif** - Show both TUI (terminal) and browser dashboard updating in real-time
2. **breeding.gif** - Show the geometric constraint solver routing requests to optimal species
3. **memory.gif** - Show CRDT memory being written and synced across sessions
4. **install.gif** - One-command install on fresh Jetson

## Dimensions

- Width: 1200px recommended
- Height: 800px recommended
- Format: GIF, max 5MB each
- Frame rate: 15-30 fps

## Placeholder Files

Until real screenshots are captured, the README uses text-based diagrams instead of actual GIFs.
