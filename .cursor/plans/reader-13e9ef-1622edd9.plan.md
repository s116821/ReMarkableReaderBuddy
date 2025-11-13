<!-- 1622edd9-adb4-4758-b83f-09aeb9994ab3 42a95dc9-e0aa-4708-8a50-3470e8117975 -->
# Reader Buddy Bugfix Plan

## Steps

1. Capture Useful Iteration Context

- Update `src/workflow/orchestrator.rs` to retain the decoded screenshot (as grayscale) alongside `AnalysisResult` so rendering/erasure logic can inspect original pixels.
- Add targeted debug logging (guarded by verbose log level) to record transformed coordinates used for erasing, symbol placement, and page swipes to aid future calibration.

2. Improve Question Erasure Accuracy

- Extend `Workflow::erase_region` (and supporting code in `src/device/pen.rs`) to accept a "smart" mask: crop the stored screenshot to the LLM `QUESTION_BOX`, threshold ink pixels, and drive eraser strokes only across detected ink segments (with a modest margin) rather than a fixed rectangle.
- Clamp all generated coordinates to the 768×1024 virtual workspace to avoid stray eraser streaks.

3. Make Symbols & Answer Rendering Visible

- Replace the placeholder circle in `src/workflow/symbol_pool.rs` with an actual rendered glyph by generating a tiny SVG (leveraging `ghostwriter::util` patterns) and drawing it via the existing bitmap path; ensure size/contrast is adjustable via constants.
- After drawing the symbol, insert a short delay (or explicit sync) before page navigation so the mark settles on e-ink displays.

4. Invoke Native New-Page Creation

- Investigate xochitl’s `LateralMenu` DBus utilities (or documented `/sys/devices/virtual/input` commands) to trigger the native "add page to the right" action instead of gestural swiping, wrapping it in a new helper under `src/workflow/page_manager.rs`.
- Wait for confirmation (e.g., by polling xochitl’s DBus status or observing page-count signals) before typing the answer to ensure the new page exists and is active.

5. Verification & Docs

- Add a `--debug-dump` CLI flag in `src/main.rs` to optionally save the marked-up screenshot plus mask overlays for off-device review.
- Document the new behavior and troubleshooting tips (calibration, debug flag) in `README.md` and add a note on the smarter erasure approach.

### To-dos

- [ ] Store iteration screenshot/metadata for downstream rendering steps
- [ ] Implement adaptive question erasure using pixel thresholding
- [ ] Render legible circled-number glyphs and sync before navigation
- [ ] Strengthen swipe gesture reliability and ensure new page is active before typing
- [ ] Expose optional debug dumps and document updated behavior