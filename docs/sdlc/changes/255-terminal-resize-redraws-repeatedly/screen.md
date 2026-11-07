# Screen: terminal pane during and after a window resize

No layout, string, colour, or control changes. Only *when* the terminal reflows
changes. Window width 1280px → 900px, one session pane.

## Before — every intermediate size is sent

```
frame t0   1280px  ┌ claude ─────────────────────────────────────┐ agent redraw #1
frame t1   1210px  ┌ claude ───────────────────────────────┐       agent redraw #2
frame t2   1100px  ┌ claude ───────────────────────────┐           agent redraw #3
 …                  …                                               …
frame tN    900px  ┌ claude ────────────────────┐                  agent redraw #N
```

## After — sent once the size has held still for 150 ms

```
during drag        ┌ claude ────────────────────┐   last capture stays, clipped to the
 (size changing)   │ > 前回のサイズで折り返した…│   pane (wider: blank to the right);
                   └────────────────────────────┘   no agent redraw
150 ms after stop  ┌ claude ────────────────────┐   agent redraw ×1 at the final size
                   │ > 新しい幅で折り返し済み   │
                   └────────────────────────────┘
```

- **Maximize / restore**: the animation's frames are one gesture → one redraw.
- **Minimize to Dock and back**: a size that flickers and returns within 150 ms is never sent.
- **New session (no size sent yet)**: sent immediately, as before.
- **Empty / waiting state** (「ターミナル出力を待っています…」): unchanged.
