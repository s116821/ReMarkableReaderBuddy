# Technical Documentation

## Architecture Overview

ReMarkable Reader Buddy is a Rust application that provides AI-powered reading assistance on reMarkable tablets. It detects outlined content and handwritten questions, then displays answers on new pages.

### Project Structure

```
src/
├── main.rs              # Application entry point with CLI
├── lib.rs               # Library exports
├── device/              # Hardware interaction (from ghostwriter)
│   ├── mod.rs           # Device detection (RM2 vs Paper Pro)
│   ├── screenshot.rs    # Framebuffer capture
│   ├── pen.rs           # Drawing via evdev
│   ├── keyboard.rs      # Virtual keyboard input
│   └── touch.rs         # Touch event handling
├── llm/                 # LLM integration
│   ├── mod.rs           # LLM trait
│   └── openai.rs        # ChatGPT API client
├── analysis/            # Image analysis
│   ├── mod.rs           # Types (QuestionContext, BoundingBox)
│   ├── circle_detector.rs     # Outline detection (placeholder)
│   └── question_extractor.rs  # Question extraction (placeholder)
└── workflow/            # Orchestration
    ├── mod.rs           # Main workflow coordinator
    ├── orchestrator.rs  # High-level control flow
    ├── page_manager.rs  # Page navigation (TODO)
    └── renderer.rs      # Content rendering (placeholder)
```

## Current Workflow (v0.1)

1. User outlines content (any closed shape)
2. User writes question nearby
3. User touches lower-right corner
4. App captures screenshot (768x1024)
5. **Single LLM Call**: Detects outline + extracts question + generates answer
6. **[TODO]** Create new blank page after current
7. Render Q&A on new page
8. **[TODO]** Erase question text (preserve outline)
9. **[TODO]** Draw reference symbol on both pages

### Why Single LLM Call?

**Design Decision**: One vision API call handles everything (detection + OCR + answering) because:
- **Simpler**: Less code, fewer edge cases
- **Faster**: Saves one API round-trip (5-10 seconds)
- **Cheaper**: One API call instead of two
- **Context**: LLM sees full context while answering
- **Effective**: GPT-4o vision handles all tasks well

**Note**: reMarkable has native handwriting conversion (MyScript engine), but it's not easily accessible from third-party apps. The LLM vision approach is more practical and works well.

## Key Technical Details

### Device Detection

Reads `/etc/hwrevision` to determine:
- reMarkable 2: `armv7`, 1872x1404, 16-bit grayscale
- Paper Pro: `aarch64`, 1632x2154, 32-bit RGBA

### Screenshot Capture

- Finds `xochitl` process PID
- Reads framebuffer from `/proc/{pid}/mem`
- Applies color correction and orientation
- Normalizes to 768x1024 virtual resolution

### Pen Drawing

- Uses `/dev/input/event1` (RM2) or `/dev/input/event2` (RMPP)
- Simulates pen events via evdev
- Supports lines, rectangles, and bitmap rendering

### Touch Detection

- Monitors `/dev/input/event2` (RM2) or `/dev/input/event3` (RMPP)
- 68x68 pixel trigger zones in corners
- Default: Lower-right (LR)

### Virtual Keyboard

- Creates uinput virtual device
- Maps characters to key events
- Supports Ctrl commands for text formatting

## High Priority TODOs

### 1. Page Management System

**Goal**: Insert new blank page after current page

**Research Needed**:
- How does xochitl manage pages?
- File system structure for documents
- IPC mechanisms available
- Page navigation triggers

**Approaches to Investigate**:
- Direct file system manipulation
- xochitl process signals
- D-Bus IPC (if available)
- Touch gesture simulation for page creation

### 2. Outline Preservation

**Goal**: Erase only question text, keep outline intact

**Required Changes**:

Update `QuestionContext`:
```rust
pub struct QuestionContext {
    pub outlined_content_region: BoundingBox,  // Don't erase
    pub question_region: Option<BoundingBox>,  // Erase this
    pub question_text: String,
    pub full_screenshot_base64: String,
}
```

Update LLM prompt to return TWO regions:
- OUTLINE_REGION: The drawn shape (preserve)
- QUESTION_REGION: The text (erase)

### 3. Symbol Pool Implementation

**Goal**: Reference markers ①②③④⑤⑥⑦⑧⑨⑩

**Requirements**:
- Pool of 10 distinct symbols
- Cycle through them across triggers
- Persist symbol counter across app restarts
- Render clearly at small size
- Place on both source and answer pages

**Rendering Options**:
- SVG to bitmap conversion
- Font-based rendering (if reMarkable fonts available)
- Pre-rendered bitmap glyphs

### 4. Question Erasure

**Goal**: White-fill question region only

**Testing Needed**:
- Verify white rectangle erases on device
- Ensure it doesn't affect outline
- Handle different pen colors/thicknesses

## Dependencies

Core crates:
- `evdev` - Input device simulation
- `image` + `imageproc` - Image processing
- `ureq` - HTTP client for OpenAI API
- `clap` - CLI parsing
- `serde_json` - JSON handling
- `base64` - Image encoding
- `anyhow` - Error handling
- `log` + `env_logger` - Logging

## Building

### Cross-Compilation

```bash
# Requires Docker and cross tool
./build.sh rm2    # armv7 for reMarkable 2
./build.sh rmpp   # aarch64 for Paper Pro
```

### Local Testing

Cannot run natively on Windows/Linux desktop (requires reMarkable hardware). Use:
- `--input-png` for testing with sample images
- `--no-draw` for testing without device output
- `--save-screenshot` for debugging

## CI/CD Integration

Uses **MagDrago Rust Semver Action** for automated versioning:

**Version Bump Rules**:
1. **Major** (X.0.0): Commit with `!` before `:`
2. **Minor** (0.X.0): Merge from `feature/` branch
3. **Patch** (0.0.X): Any source file change
4. **None**: Docs-only changes

**Source Files**: Defined in `.versioning/source_globs.txt`

**Workflows**:
- `.github/workflows/ci.yml` - Format, lint, check on PRs
- `.github/workflows/release.yml` - Version, build, release on main

## Testing Strategy

### Current Testing
- Manual device testing required
- Use `--input-png` for offline testing
- Check logs with `--log-level debug`

### Future Testing
- Unit tests for analysis algorithms
- Integration tests with sample screenshots
- Mock device interfaces for desktop testing

## Performance Considerations

- LLM vision calls: 5-10 seconds latency
- Screenshot capture: ~100ms
- Pen drawing: Depends on complexity
- Future: Local CV could reduce LLM calls

## Known Limitations (v0.1)

1. Single outline-question pair per trigger
2. No page creation (renders to current page)
3. Simple geometric symbols (not ①②③ yet)
4. No symbol cycling/tracking
5. No context retention between triggers
6. LLM-based detection (expensive, slow)
7. Requires internet connection

## Extension Points

### Adding LLM Providers

Implement `LLMEngine` trait in `src/llm/`:
```rust
pub trait LLMEngine {
    fn add_text_content(&mut self, text: &str);
    fn add_image_content(&mut self, base64_image: &str);
    fn clear_content(&mut self);
    fn execute(&mut self) -> Result<String>;
}
```

### Adding Local CV Detection

Implement in `src/analysis/circle_detector.rs`:
- Hough Circle Transform
- Contour detection
- Shape analysis
- Fallback to LLM if fails

## Troubleshooting

### Build Issues
- **Windows**: Cannot build natively (ARM targets only)
- **Solution**: Use `cross` with Docker

### Runtime Issues
- **No xochitl process**: Tablet in sleep mode or no document open
- **Touch not working**: Check trigger corner setting, use hand not pen
- **API errors**: Verify OPENAI_API_KEY is set

## References

- **ghostwriter**: Core device interaction code source
- **MagDrago Rust Semver Action**: Automated versioning
- **reMarkable Community**: Device documentation
- **OpenAI**: Vision API capabilities

---

**Version**: 0.1.0  
**Last Updated**: 2025-11-01

