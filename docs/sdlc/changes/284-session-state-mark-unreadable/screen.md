# Screen: the state badge holds the state's mark

Tab strip, before and after:

```
Before:  [✳]• セッション5 ×    [A]• セッション2 ×    [>_] セッション6 ×
After:   [✳](‖) セッション5 ×  [A](✓) セッション2 ×  [>_] セッション6 ×
              idle badge          done badge          running terminal: none
```

Left column, before and after:

```
Before                               After
p  ⚠1 ⟳2                    +        p  (!)1 (⟳)2              +
  [✳]•  セッション5                     [✳](‖) セッション5
  ⟳ my-cmd  セッション9                 (⟳) my-cmd  セッション9
```

- The badge sits on the mark's lower right corner as the dot did, larger
  (about 9px), filled with the state's colour, its glyph in the tab's or
  row's fill colour, ringed in that fill.
- Project counts: the generated circled mark in the group's colour, then the
  count; no font glyph.
- Nothing is added, moved or removed; only marks change.
