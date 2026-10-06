# Screen: adversarial hardening for sidebar splitters, drag drop bounds, and ID isolation

## Before

When docked to the right side, dragging the splitter collapses the sidebar to 180px and prevents resizing. Drag drop can trigger on non-Primary button release or lack Escape cancellation.

```
(Right side dock)
┌──────────────────────────────────────────┬────────────────────────┐
│ A .tmpにテスト生成 ○IDLE · operon        │ │[ファイル] [会話]   ⟩ │
│ ┌terminal──────────────────────────────┐ │ │                      │
│ │                                      │ │ │ (Collapses to 180px  │
│ │                                      │ │ │  when splitter dragged│
│ │                                      │ │ │  due to remaining    │
│ │                                      │ │ │  width calculation)  │
│ └──────────────────────────────────────┘ │ │                      │
└──────────────────────────────────────────┴─┴──────────────────────┘
                                           ▲
                                           │ (Splitter clamped to 180.0)
```

## After

Both left and right sidebars can be smoothly resized between 180px and 480px. Dragging a tab shows zero frame jitter, has a 24px deadzone around the center line to prevent accidental switching, and can be cancelled with `Escape`.

```
┌──────────────────────────────────────────┬────────────────────────┐
│ A .tmpにテスト生成 ○IDLE · operon        │ │[ファイル] [会話]   ⟩ │
│ ┌terminal──────────────────────────────┐ │ │                      │
│ │                                      │ │ │ (Smoothly resizes    │
│ │                                      │ │ │  between 180px and   │
│ │                                      │ │ │  480px symmetrically)│
│ └──────────────────────────────────────┘ │ │                      │
└──────────────────────────────────────────┴─┴──────────────────────┘
                                           ▲
                                           │ (Splitter symmetrically clamped)
```

## States that are not the happy one

| State | Shows |
|---|---|
| User presses Escape while dragging | Drag tooltip and grabbing cursor dismiss immediately; side remains unchanged |
| User releases right-click or middle-click during drag | Drag continues uninterrupted; only Primary button release triggers drop |
| User drops within 24px of screen center | No side switch occurs (deadzone protection against accidental drop) |
| User drops outside window / panel | No side switch occurs (boundary containment check) |
