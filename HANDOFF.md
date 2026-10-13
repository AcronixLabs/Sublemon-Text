# Handoff

## Current objective

Continue tuning Sublemon's clean-room UI to match `sublime_goal.png` as closely as practical. The user wants `HANDOFF.md` and `SESSION_LOG.md` updated at the end of each work step/session.

## Current implementation state

- The fake text-based minimap is removed.
- The sidebar remains draggable; its panel ID is `sidebar_layout_v2` to discard the old saved width. Its new initial width targets 43% of the viewport, clamped to 250–550 egui points.
- The project sidebar minimum width is zero, so the drag handle can shrink it down to the divider.
- The project tree has custom-painted folder, Markdown, Rust, image, configuration, and generic text-file icons; folder disclosure markers are painted chevrons.
- The line-number gutter is laid out beside the editor under the tab strip. Tabs begin at the sidebar edge, as in the reference.
- Markdown is wrapped and lightly highlighted.
- `cargo fmt --check` and `cargo check --offline` passed after these changes.

## Screenshot status

- The user supplied a new 1920×1080 screenshot. It shows the corrected tab/gutter arrangement and a working editor.
- In that screenshot the sidebar occupies about 34% of the width; the reference sidebar occupies about 43%. The code now targets 43% on first layout for the new panel ID. Capture again to verify.
- The tree icons and zero-width sidebar change have compiled, but they have not yet been visually captured.
- The reference file `sublime_goal.png` is 941×1038. Resize Sublemon to the reference's app-window dimensions before a meaningful pixel comparison.
- A local app capture taken before the sidebar-ID reset showed an oversized sidebar. The attempted capture after the reset landed on a different workspace and is invalid. Do not treat `/tmp/sublemon-after-layout.png` as a verified post-reset app screenshot.

## Next actions

1. Relaunch Sublemon and capture the actual app window to verify the tree icons and the zero-width sidebar drag behavior.
2. Resize the app to 941×1038 to match `sublime_goal.png` and use `scripts/ui_screenshot_test.py` to quantify similarity. The goal previously discussed is at least 80% pixel similarity.
3. Prioritize remaining visible layout mismatches (sidebar proportion, tab sizing/placement, gutter alignment, and line wrapping), then update both handoff files at the end of that work step.

## Clean-room constraint

Use only independently authored code/assets and public functional descriptions. Do not inspect or use the installed Sublime Text files, binaries, extracted resources, or disassembly.
