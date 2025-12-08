# Image Comparison Masks

This document explains the image masking system used during page comparisons in Reader Buddy.

## Overview

When comparing screenshots to detect page changes or verify navigation, Reader Buddy masks out certain screen regions to avoid false positives caused by transient UI elements. Without these masks, temporary UI popups and toolbar states could cause the system to incorrectly detect page changes.

All mask values are in **virtual coordinates** (768×1024 pixels). Screenshots from both reMarkable 2 and reMarkable Paper Pro are normalized to this size before comparison.

## Mask Regions

### Left Side Mask (`MASK_LEFT_OFFSET`: 298 pixels)

**Purpose:** Avoid the left toolbar and its transient popups.

The reMarkable UI typically displays a toolbar on the left side of the screen. This toolbar can:
- Expand/collapse showing different icons
- Display brief dialog popups when tools are selected
- Show tooltips or selection highlights

By masking the left 298 pixels, we ignore any toolbar state changes that could affect page comparisons.

### Right Side Mask (`MASK_RIGHT_OFFSET`: 125 pixels)

**Purpose:** Avoid the close button, page controls, and occasional popups.

The right side of the screen may contain:
- The **X button** (close/exit button) in the top-right area
- The **"Add new page" popup** that can briefly appear
- Scrollbar or page navigation indicators
- Other transient UI elements

By masking the right 125 pixels, we prevent these elements from affecting similarity calculations.

### Top Mask (`MASK_TOP_OFFSET`: 70 pixels)

**Purpose:** Avoid the toolbar when docked at the top of the screen.

Users can configure the reMarkable toolbar to be docked at the top of the screen instead of the left side. When docked at the top:
- The toolbar takes up vertical space at the top
- Tool selection states can change the appearance
- Brief popups or tooltips may appear

By masking the top 70 pixels, we handle both toolbar positions consistently.

### Bottom Mask (`MASK_BOTTOM_OFFSET`: 70 pixels)

**Purpose:** Avoid the page navigation HUD and status popups.

The bottom of the screen can display:
- **Page navigation HUD** showing current page number
- **Swipe indicators** when navigating between pages
- **Status messages** or brief notifications
- **Loading indicators**

These elements appear and disappear dynamically, so masking the bottom 70 pixels prevents them from causing false page-change detections.

## Visual Representation

```
+----------------------------------------------------------+
|████████████████████ TOP MASK (70px) █████████████████████|
+------+-------------------------------------------+-------+
|██████|                                           |███████|
|██████|                                           |███████|
|██████|                                           |███████|
|██ L █|                                           |██ R ██|
|██ E █|         COMPARISON REGION                |██ I ██|
|██ F █|                                           |██ G ██|
|██ T █|      (Only this area is compared)        |██ H ██|
|██████|                                           |██ T ██|
|██298█|                                           |██125██|
|██ px█|                                           |██ px██|
|██████|                                           |███████|
|██████|                                           |███████|
+------+-------------------------------------------+-------+
|████████████████ BOTTOM MASK (70px) ██████████████████████|
+----------------------------------------------------------+
```

## Usage in Code

The masks are defined as public constants in `src/workflow/mod.rs`:

```rust
pub const MASK_LEFT_OFFSET: u32 = 298;   // Left toolbar
pub const MASK_RIGHT_OFFSET: u32 = 125;  // Right side UI
pub const MASK_TOP_OFFSET: u32 = 70;     // Top docked toolbar
pub const MASK_BOTTOM_OFFSET: u32 = 70;  // Bottom HUD
```

These are passed to `compute_image_similarity_masked()` which skips pixels in the masked regions during comparison.

## When Masks Are Applied

Masks are applied during:
1. **Page navigation detection** - Checking if we moved to a different page
2. **Return-to-original verification** - Confirming we navigated back to the question page
3. **Blank page detection** - Checking if a page is empty (compares against synthetic white image)
4. **QA header detection** - Checking if a page has the Reader Buddy answer header

## Tuning Considerations

If you encounter issues with page comparison (false positives or negatives), consider:
- **Increasing mask sizes** if UI elements are still causing issues
- **Decreasing mask sizes** if too much content is being ignored
- **Adjusting similarity thresholds** in conjunction with mask changes

The current values were tuned for standard reMarkable UI configurations and should work for most use cases.

