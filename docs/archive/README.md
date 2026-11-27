# Documentation inventory and historical index

The audit covered the project README and the 102 pre-existing files in `docs/`,
including JSON measurement records and GTK patch files. These categories assign
a purpose to each group. Historical records keep their original URLs where they
serve as references in old commits, packaging or test procedures.

| Location or file group | Classification and use |
| --- | --- |
| Project README and documentation index | Current user entry points. The README separates v0.2.7, the development snapshot and planned work. |
| `architecture/` | Current source map and persistent thumbnail freshness contract. |
| `development/` | Current contribution workflow and chronological development notes. Commands and results within dated entries apply to those entries. |
| `testing/` and `packaging/` | Current validation and build workflow. For packaging operations use the guide under `docs/`; existing README files under the source `packaging/` tree include earlier version-specific instructions. |
| `diagnostics/OPEN_FILE_LATENCY.md`, `THUMBNAIL_REQUEST_DIAGNOSTICS.md`, `VIDEO_ADAPTER_REBUILD.md` | Current investigation procedures, observation limits and adapter source/relink instructions. |
| `diagnostics/GNOME_FACTORY_*.md` and `VIDEO_COLOR_*.md` | Incident, design and controlled experiment records. Earlier CPU, custom GTK and opt-in backend prototypes are superseded by stock GTK application integration. Their measurements remain historical. |
| `performance/` | Historical v0.2.6 measurements, JSON outputs and isolated GTK patch reviews. The upstream ownership handoff remains relevant. Keep the patches and their matching test runners at their tracked paths. |
| `probes/` | Historical PIV-05 and PIV-09 machine-readable environment and decoder evidence. |
| `slideshow/`, `ui/` | Historical feature-branch, EOS and toolbar design records for v0.2.7. Current behavior belongs in the source map and release notes. |
| `upstream/` | Current reference for the open GTK Wayland EGL issue, with historical reproductions and isolated patch evidence. |
| `releases/` | Release policy and v0.2.8 draft/validation are current. The v0.2.7 notes and validation establish the published stable scope and its recorded evidence. All earlier versioned documents are historical, including v0.2.5 and the unreleased v0.2.6 candidate. |
| Root `PIVRA_FEATURE_SPEC_TR.md`, `PIVRA_DEVELOPMENT_ROADMAP.md` | Product decisions and planned work. The roadmap also contains dated milestones. |
| Root `PIVRA_COMMIT_TRACKER.md`, `PIVRA_TECHNICAL_CAPABILITIES.md` | Historical milestone and environment evidence. Use the current guides for supported workflows. |
| Root `FD*.md`, `PIV09_*.md`, `PIV1*.md`, `TH0*.md` | Historical provider, rendering and thumbnail milestones, plans, and validation. Retained for design rationale and reproducibility. |
| Root `PHASE2_RELEASE_READINESS.md`, `V0_1_PHOTO_NAVIGATION.md`, `WINDOW_STATE_VALIDATION.md`, `THUMBNAIL_SCROLLBAR_VALIDATION.md` | Historical readiness, photo-only navigation, window state and scrollbar evidence. The early photo-only design predates mixed-media navigation. |

The [v0.2.8 validation report](../releases/PIVRA_V0.2.8_RC_VALIDATION.md) is the
current test-results authority. Historical test commands, paths and package
instructions refer to their recorded source revisions. In particular,
`src/api_loader.rs`, `src/pool.rs` and `src/fdct.rs` mentioned in those records
are upstream dependency paths from their respective versions.

The audit grouped troubleshooting records under Diagnostics and moved the
thumbnail cache contract under Architecture. It retained milestone files and
historical numerical evidence. No archives, reports or patch files were removed.

The review found three classes of maintenance issues:

- Older Phase 2, slideshow and GPU prototype notes used present-tense release
  status. Context banners and the index date these records and keep the older
  evidence intact.
- The source packaging README files duplicate earlier release instructions and
  do not describe one consistent target. The current guide identifies the
  0.2.6 validator versus 0.2.7 recipe and staging mismatch as future release work.
- Current source responsibilities and contributor, test and release workflows
  lacked one linked reference. The new guides cover these subjects and point
  to implementation notes only when historical detail is relevant.

The link audit found no missing local Markdown targets before the move. Relative
links to the moved cache contract were updated and rechecked after the move.
