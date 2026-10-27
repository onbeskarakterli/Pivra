# Thumbnail scrollbar: paint and hit-area regression

This is a separate UI change after the thumbnail request/retry commit.

## Proven cause and minimal fix

The previous CSS set horizontal scrollbar opacity to zero while the
ScrolledWindow used Automatic policy. A transparent widget can remain mapped
and receive clicks/drags; opacity is not an input-region removal mechanism.

Use External horizontal policy: keep the scrollable adjustment/content extent
without an internal scrollbar. Explicitly hide the internal horizontal scrollbar,
set can_target(false) and focusable(false), and remove the opacity-only CSS.
Vertical policy, overlay layout, height preference, resize edge, existing wheel
and left-drag controllers, thumbnail selection and image transforms are unchanged.
No external scrollbar, toolbar or replacement scrolling engine is added.

## Separate controlled GTK regression

`thumbnail_overlay_probe --thumbnail-scrollbar-regression` uses real
ScrolledWindow, adjustment, widgets, production event controllers and hit tests,
with private cached TEST media and synthetic pixels. It asserts:

- External policy; scrollbar invisible, unmapped and non-targetable.
- Bottom-band GTK pick results never address the scrollbar or its descendants.
- Vertical wheel and horizontal/fractional touchpad-axis callbacks move the
  existing horizontal adjustment; this is not physical device-input proof.
- Left-button drag changes that adjustment; a zero-motion drag does not prevent
  thumbnail selection callback delivery.
- The main viewport size/pixels and strip requested/allocated height are stable;
  resize retains the requested height and no scrollbar target reappears.

Headless Sway driver entry:
`tests/wayland_smoke.py <thumbnail_overlay_probe> --navigation-probe --thumbnail-scrollbar`.
No assertion from the prior catalog/retention tests was removed or relaxed.

## Results and native acceptance

Controlled X11/Xvfb and headless Sway scrollbar regressions: PASS. Current Rust
fmt/check/test and related catalog/81-and-120 retention results are recorded in
the delivery report. Synthetic GTK events are not physical GNOME acceptance.

On Arch/GNOME with the updated binary, open a private test folder, show/pin the
strip, and verify no bottom scrollbar or invisible thumb/track intercepts input.
Check mouse wheel, touchpad horizontal/vertical smooth scrolling, left-drag,
first thumbnail click and zero-motion click; drag the upper resize edge and
confirm proportional cells/saved height. Test narrow/wide windows, two windows,
Fit/Fill/100%, zoom/pan, show/hide and native theme. No source images or user
cache should be deleted. **Physical GNOME/touchpad acceptance remains NOT RUN
in Codex.**

Final local run: fmt/check PASS, 190 Rust tests PASS, 41 Python thumbnail/helper
tests PASS; all-target Clippy exit 0 with the pre-existing load_stats/pin-icon
warnings. New scrollbar regression PASS in X11 and Sway. Existing controlled
catalog and 81/120 retention PASS; 48 synthetic Fit/Fill/Actual snapshots and
zoom/pan regression PASS in X11. No physical device/native decoder PASS claimed.
