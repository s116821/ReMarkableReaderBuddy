<!-- a468eca1-0c43-4cfb-84c4-cf05e2c701c5 bcf7f5e1-b1f7-402c-8a25-e899e4367a10 -->
# Simplify Answer Page Flow

> **Extends**: [Reader Buddy Bugfix Plan](reader-13e9ef-1622edd9.plan.md)
>
> **Relationship**: This plan pivots away from the original implementation approach. The original plan attempted to use xochitl's toolbar/menu interactions and DBus utilities to automatically create new pages when needed. This proved unreliable due to varying menu coordinates, timing issues, and device-specific behavior. This plan simplifies the workflow by **removing all auto page creation** and instead requiring the user to pre-place a blank or QA page to the right. The tool now only navigates once and draws a failure X if no suitable page exists, rather than attempting complex menu interactions.

1) Remove auto page creation

- Strip toolbar detection and page-creation calls from the workflow (orchestrator render path and page_manager/xochitl_integration). Only navigate right once and never attempt menu interactions.

2) Failure handling via drawn X

- After attempting to swipe right: if no page or the page is not blank/QA, return/stay on the origin page and draw a large failure X (~150x150 px) in the bottom-right (using pen draw, no text output). Ensure no other text/progress is shown on the origin page.

3) Detection logic update

- Update the answer-page check to accept either an existing QA header or a blank page; if neither, treat as failure. Keep the cached header match for QA pages and add a simple blank-page ink-threshold check.

4) Documentation

- Update README (usage section) to explain the new expectation: user must place a blank or QA page to the right; the tool will only render on that page and will mark an X on the origin page if unavailable.

### To-dos

- [ ] Remove auto page creation and toolbar/menu usage
- [ ] Implement failure X on origin page bottom-right (~150px)
- [ ] Adjust page check: accept blank or QA header
- [ ] Document new workflow/expectations in README