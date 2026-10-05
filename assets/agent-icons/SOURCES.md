# Agent icons

The icon macOS shows for each CLI's vendor app, drawn beside its name so a
session card names an agent the way a person already recognises it from their
Dock. The marks are third-party trademarks used to identify the tool Operon
launches; they are not part of this project's own branding and stay unmodified
apart from the framing below.

They are **not** covered by this repository's MIT License; see
`THIRD_PARTY_NOTICES.md`. A rights holder who wants one removed should say so
in an issue, and it will be replaced with a generic glyph.

| File | Agent | Taken from |
| --- | --- | --- |
| `claude-code.png` | Claude Code | `/Applications/Claude.app/Contents/Resources/electron.icns` |
| `codex.png` | Codex CLI | `/Applications/ChatGPT.app/Contents/Resources/icon-codex-light.png` |
| `antigravity.png` | Antigravity CLI (`agy`) | `/Applications/Antigravity.app/Contents/Resources/icon.icns` |

Take the app icon, not the bare logotype a vendor also ships. Every macOS app
icon is the same rounded square, so three of them line up as one row; three
loose logotypes are three silhouettes at three optical weights, and the odd one
out is what a person notices instead of the label they were reading.

Codex is the only vendor here that ships a plate per appearance. Take the light
one: `icon-codex-dark-color.png` is a near-black tile that sinks into the dark
panel, and the point of the plate is that it reads as the same chip on every
theme.

## Regenerating

Each file is the vendor icon cropped to the rounded square itself, centred in a
square canvas, and resized to 128x128. An `.icns` source needs
`sips -s format png <src>.icns --out <src>.png` first.

Crop on the **opaque** pixels, not on the alpha bounding box. Claude and Codex
ship their icon with the soft drop shadow macOS draws under a Dock tile, and
that shadow is 2% of the canvas wider and sits low; trimming to it would leave
their plates smaller than Antigravity's shadowless one and nudged up inside the
box, which is the misalignment this framing exists to remove.

```python
from PIL import Image

def normalize(src, dst, size=128):
    image = Image.open(src).convert("RGBA")
    plate = image.split()[3].point(lambda a: 255 if a >= 250 else 0)
    image = image.crop(plate.getbbox())
    width, height = image.size
    side = max(width, height)
    canvas = Image.new("RGBA", (side, side), (0, 0, 0, 0))
    canvas.paste(image, ((side - width) // 2, (side - height) // 2))
    canvas.resize((size, size), Image.LANCZOS).save(dst, optimize=True)
```
