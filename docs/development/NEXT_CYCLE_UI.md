# Post-v0.2.7 viewer improvements

Development on `dev`; version remains 0.2.7. No release preparation.

## Empty-window Open

Open now uses the same GTK FileDialog and `open-file` URI action in empty and
active windows. Each window initializes one viewer/context; empty controls stay
hidden until admission. Selecting a file stops the old watcher/slideshow, reuses
the viewer and installs the new folder monitor. Cancel leaves selection intact;
close/destroy cancels an outstanding chooser.

Validation: formatting and locked compilation PASS. Controlled GTK empty/open/
reopen/same-window/new-folder-monitor/navigation gate PASS on X11. Actual native
GNOME file-picker selection/cancel and secure decoding: **NOT TESTED** here.
