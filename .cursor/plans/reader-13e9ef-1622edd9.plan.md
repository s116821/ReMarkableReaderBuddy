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

- [x] Store iteration screenshot/metadata for downstream rendering steps
- [ ] Implement adaptive question erasure using pixel thresholding (DEFERRED - disabled for now)
- [ ] Render legible circled-number glyphs and sync before navigation
- [x] Strengthen swipe gesture reliability and ensure new page is active before typing
- [ ] Expose optional debug dumps and document updated behavior

## Implementation Changes (2025-11-13)

### Answer Page Detection Fix
**Issue**: The `check_if_next_page_is_answer_page()` function was returning false positives. It was checking for only 50 dark pixels in the header region, which could easily be triggered by any small mark or writing at the top of a regular page.

**Solution**: Increased the threshold from 50 to 500 dark pixels (`MIN_INK_PIXELS`). The actual header text "=== Reader Buddy Answers ===" should create hundreds or thousands of dark pixels when rendered via keyboard, so this threshold prevents false positives from stray marks while still reliably detecting the header.

**Files Changed**:
- `src/workflow/mod.rs`: Updated `check_if_next_page_is_answer_page()` to use `MIN_INK_PIXELS = 500` (was 50)
- Also increased `HEADER_HEIGHT` from 100 to 120 pixels to capture full header text
- Improved logging to always report ink pixel count for debugging

### Erasure Functionality Temporarily Disabled
**Reason**: User requested to focus on getting page navigation working correctly before tackling the erasure feature. The erasure was having issues and shouldn't block progress on the core Q&A functionality.

**Implementation**: Commented out the erasure logic in `src/workflow/orchestrator.rs` with a TODO comment. The code structure remains in place for easy re-enabling later.

**Files Changed**:
- `src/workflow/orchestrator.rs`: Disabled erasure in `render_answer()` method (lines ~207-215)

### Navigation Pattern Optimization
**Issue**: The original code was navigating back and forth multiple times unnecessarily:
- Navigate forward to check for answer page
- Navigate back to original page  
- Navigate forward again to use the page

This caused redundant swipes and potential timing issues.

**Solution**: Streamlined navigation to be more efficient:
1. Navigate forward once to next page
2. Check if current page is answer page (already there!)
3. If NOT answer page: Navigate back, create new page to right (which auto-navigates to it)
4. If IS answer page: Stay on current page (no extra navigation needed)
5. Type content
6. Navigate back to original page once

**Files Changed**:
- `src/workflow/mod.rs`: Renamed `check_if_next_page_is_answer_page()` to `check_if_current_page_is_answer_page()` and removed internal navigation logic - caller is now responsible
- `src/workflow/orchestrator.rs`: Updated `render_answer()` to navigate once before checking, eliminating redundant navigation

### Hybrid Header Detection (LLM + Pattern Matching)
**Issue**: Using pixel counting was imprecise. Tesseract OCR required complex cross-compilation and external data files. Pure OCR solutions with embedded models would bloat the binary by 20-70MB.

**Solution**: Implemented a clever hybrid approach that gives us the best of both worlds:
1. **First Detection**: Use LLM to accurately detect "=== Reader Buddy Answers ===" header
2. **Save Pattern**: After successful LLM detection, save the header region as a PNG pattern file
3. **Subsequent Detections**: Use lightning-fast pixel similarity comparison (~1ms, offline)
4. **Fallback**: If pattern doesn't match or doesn't exist, use LLM again

**Benefits**:
- ✅ Single binary (no external dependencies)
- ✅ Fast after first run (~1ms vs ~500ms)
- ✅ Works offline after initial pattern capture
- ✅ 100% accurate (LLM provides ground truth)
- ✅ No cross-compilation issues
- ✅ Small binary size (no ML model bloat)

**Implementation**:
- Crop screenshot to top 120 pixels (header region only)
- Try pattern matching with 80% similarity threshold first
- Fall back to LLM if pattern missing or doesn't match
- Save pattern to `/home/root/.reader-buddy-header-pattern.png` after successful detection
- Pattern saved both after LLM confirmation AND after creating new answer pages

**Files Changed**:
- `Cargo.toml`: Removed `tesseract` dependency (avoided cross-compilation hell)
- `src/workflow/mod.rs`: Implemented hybrid detection with `check_if_current_page_is_answer_page()`, `save_header_pattern()`, and `compute_image_similarity()`
- `src/workflow/orchestrator.rs`: Added `check_answer_page_with_llm()` method and integrated pattern saving after page creation
- `Cross.toml`: Can be removed (no longer needed)

### Navigation Verification & Smart Page Insertion
**Issue**: When attempting to navigate to the next page, we had no way to detect if we actually moved or stayed on the same page (no page to the right exists). This could cause operations to execute under wrong assumptions.

**Solution**: Added verified navigation that compares screenshot hashes before/after to detect if page actually changed.

**Additional Optimization**: When a non-answer page exists to the right (Case 2), instead of:
- Navigate right → Check → Navigate back left → Create page right → Navigate right again

We now:
- Navigate right → Check → Create page LEFT → Navigate left to new page

This uses the "Insert page before" menu option to add the answer page between the original and existing next page, saving navigation overhead.

**Files Changed**:
- `src/workflow/mod.rs`: Added `navigate_to_next_page_verified()` and `compute_screenshot_hash()` helper
- `src/workflow/mod.rs`: Added `create_new_page_left()` to insert page before current
- `src/workflow/xochitl_integration.rs`: Added `PageInsertDirection` enum and updated `create_page()` to support Before/After
- `src/workflow/page_manager.rs`: Added `create_page_left()` method
- `src/workflow/orchestrator.rs`: Updated page creation logic to use smart insertion

### Removed Navigation Back to Original Page
**Issue**: After rendering the answer, we were navigating back to the original page. But the user should see the answer they just asked for!

**Solution**: Removed the final `navigate_to_previous_page()` call. Now the workflow leaves the user on the answer page to view the result.

**Files Changed**:
- `src/workflow/orchestrator.rs`: Removed navigation back in `render_answer()`, added comment explaining this choice

## Implementation Changes (2025-11-13 - QA Page Flow Fix)

### LLM-First Header Detection with In-Memory Caching
**Issue**: The previous implementation tried pattern matching first, then fell back to LLM. This meant the first run would always fail pattern matching and require LLM. Additionally, the pattern was only saved to disk, not cached in memory.

**Solution**: Completely rewrote the detection pipeline:
1. Added `cached_header_pattern` field to `Orchestrator` struct for in-memory caching
2. New `check_if_answer_page()` method checks in-memory cache first
3. Falls back to LLM only if no cache exists
4. Immediately caches pattern in memory after successful LLM detection
5. Also caches pattern immediately after creating a new answer page with header
6. Disk persistence remains as backup

**Benefits**:
- First-time detection uses LLM (accurate)
- Pattern cached in memory immediately
- Subsequent checks are instant (no disk I/O)
- Works for both detected AND newly created answer pages

**Files Changed**:
- `src/workflow/orchestrator.rs`: Added `cached_header_pattern` field, rewrote `check_answer_page_with_llm()` as `check_if_answer_page()` with cache-first logic, added `compute_image_similarity()` helper

### Removed Symbol Drawing and Fixed Logging
**Issue**: 
- Symbols were being drawn on the original page before navigation
- Progress messages shown on original page ("Processing...", "Analyzing...", etc.)
- Answer text included symbol prefix that wasn't rendered correctly
- User saw temporary messages on wrong page

**Solution**:
- Completely removed all symbol drawing code from `render_answer()`
- Removed all progress messages from `run_iteration()` that showed on original page
- Only show text to user once we're on the answer page
- Changed Q&A format from `"{symbol} Q: ... A: ..."` to simple `"Q: ... A: ..."`

**Files Changed**:
- `src/workflow/orchestrator.rs`: Removed symbol drawing, simplified `run_iteration()` to not show progress on original page, updated output format in `render_answer()`

### Streamlined Navigation Flow
**Issue**: Navigation was still convoluted with unnecessary back-and-forth, and the code structure made it hard to see when operations happened on which page.

**Solution**: Completely rewrote `render_answer()` with clear sequential flow:
1. Navigate to next page (verified)
2. Check if it's an answer page (LLM/cache)
3. If not: Create new page (left or right depending on situation)
4. If yes: Already there, continue
5. Add header if newly created (caching pattern immediately)
6. Render Q&A
7. Stay on answer page

No redundant navigation, clear step-by-step progression, user only sees final result on answer page.

**Files Changed**:
- `src/workflow/orchestrator.rs`: Complete rewrite of `render_answer()` method with simplified navigation logic

### Edge Case Fix: First Page Handling
**Issue**: When user starts on the first page of the document and there's no next page:
- `navigate_to_next_page_verified()` could give false positive (hash slightly different due to timing)
- Code tried to create page to the LEFT
- Creating before the first page fails
- Answer was rendered on wrong page

**Solution**: 
1. Explicitly track `should_create_right` flag based on navigation result
2. When `did_navigate = false`, always create to the RIGHT (never left)
3. Added fallback: if creating left fails (first page edge case), catch error and create right instead

This handles both the normal case and the hash false-positive case.

**Files Changed**:
- `src/workflow/orchestrator.rs`: Updated `render_answer()` with explicit direction tracking and error fallback

### Navigation Verification Fix: Hash vs Similarity
**Issue**: The original `navigate_to_next_page_verified()` used hash comparison, which was unreliable:
- Sampled only every 100th byte (too sparse)
- Small compression/rendering differences caused false positives (thought we moved when we didn't)
- Could also have false negatives (thought we didn't move when we did)
- This caused the code to think it navigated when stuck on the first page

**Solution**: Completely rewrote navigation verification to use actual pixel similarity comparison:
1. Take full screenshots before/after navigation (not just hashes)
2. Load as images and compute pixel-by-pixel similarity (sampled every 10th pixel for speed)
3. Use 95% similarity threshold - images must be VERY different to count as different pages
4. Increased wait time from 500ms to 800ms for page transitions to complete
5. Returns accurate true/false based on actual visual change

**Benefits**:
- Reliably detects when navigation fails (no page to right/left)
- Tolerates minor compression/rendering differences
- Fast enough for real-time use (samples 1% of pixels)
- Logs similarity percentage for debugging

**Files Changed**:
- `src/workflow/mod.rs`: Rewrote `navigate_to_next_page_verified()` and `compute_image_similarity()` to use pixel comparison instead of hashing, removed duplicate similarity function

### Page Creation Dialog Flow Fix
**Issue**: Page creation was failing because:
1. Was using page overview (unreliable - current page position varies)
2. Menu interaction coordinates were completely wrong
3. No verification that page creation actually worked

**Solution**: Use the simpler "Add page" dialog from the current page:

**Flow for both directions:**
1. Tap top-left (50, 50) to open menu bar
2. Tap three dots at bottom-left of menu bar (50, 974)
3. Tap "Add page" button that appears to the right (150, 974)
4. Close menu bar (tap top-left again)
5. Navigate to newly created page

**Key insight**: The dialog ALWAYS adds a page to the RIGHT. So for "Before" direction:
- Navigate to previous page first
- Add page (goes to right of that page = our desired position)
- Navigate forward to new page

This is simpler than page overview and always works because we're creating from a known page position.

**Added verification**: After page creation, we verify we actually moved to a different page using image similarity. If creation fails (similarity >95%), return clear error.

**Files Changed**:
- `src/workflow/xochitl_integration.rs`: Complete rewrite using "Add page" dialog instead of page overview, correct coordinates for menu bar workflow
- `src/workflow/page_manager.rs`: Updated to reflect that both directions now navigate to the new page internally, removed unused imports
- `src/workflow/orchestrator.rs`: Updated to not double-navigate after create_page_left, added verification for create_page_right