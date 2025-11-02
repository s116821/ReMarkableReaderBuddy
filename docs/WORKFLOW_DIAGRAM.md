# CI/CD Workflow Diagram

## Complete Workflow Overview

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         DEVELOPER WORKFLOW                               │
└─────────────────────────────────────────────────────────────────────────┘

   Developer                      GitHub                      Automation
      │                              │                              │
      │  1. Create Branch            │                              │
      ├─────────────────────────────>│                              │
      │                              │                              │
      │  2. Make Changes             │                              │
      │  feat/fix/docs commits       │                              │
      │                              │                              │
      │  3. Push + Create PR         │                              │
      ├─────────────────────────────>│                              │
      │                              │                              │
      │                              │  4. Trigger CI Workflow      │
      │                              ├────────────────────────────> │
      │                              │                              │
      │                              │  5. Run Checks               │
      │                              │     • cargo fmt              │
      │                              │     • cargo clippy           │
      │                              │     • cargo check (armv7)    │
      │                              │     • cargo check (aarch64)  │
      │                              │     • cargo test             │
      │                              │ <──────────────────────────┤ │
      │                              │     ✅ CI Passed            │
      │                              │                              │
      │  6. Review & Approve         │                              │
      ├────────────────────────────> │                              │
      │                              │                              │
      │  7. Merge to main/master     │                              │
      ├─────────────────────────────>│                              │
      │                              │                              │
      │                              │  8. Trigger Release Workflow │
      │                              ├────────────────────────────> │
      │                              │                              │
      │                              │                              │
      v                              v                              v
```

## Release Workflow Detail

```
┌─────────────────────────────────────────────────────────────────────────┐
│                         RELEASE WORKFLOW                                 │
└─────────────────────────────────────────────────────────────────────────┘

Push to main
     │
     v
┌────────────────────────────────────┐
│  JOB 1: Semantic Versioning        │
│  (MagDrago Rust Semver Action)     │
└────────────────────────────────────┘
     │
     │ Analyze Commits
     ├─> feat: commits found? ───> Minor bump (0.1.0 → 0.2.0)
     ├─> fix: commits found?  ───> Patch bump (0.1.0 → 0.1.1)
     ├─> BREAKING CHANGE?     ───> Major bump (0.1.0 → 1.0.0)
     └─> docs:/chore: only?   ───> No version change, STOP
     │
     v
Update Cargo.toml version
     │
     v
Commit + Create Tag (v0.2.0)
     │
     v
Output: version_changed=true, new_version=0.2.0
     │
     v
┌────────────────────────────────────┐
│  JOB 2: Build Release Binaries     │
│  (Only if version_changed=true)    │
└────────────────────────────────────┘
     │
     ├─────────────────┬─────────────────┐
     v                 v                 v
┌─────────────┐  ┌─────────────┐  ┌──────────────┐
│ Build       │  │ Build       │  │   Package    │
│ armv7       │  │ aarch64     │  │   Binaries   │
│ (RM2)       │  │ (RMPP)      │  │   as .tar.gz │
└─────────────┘  └─────────────┘  └──────────────┘
     │                 │                 │
     └─────────────────┴─────────────────┘
                       │
                       v
            Upload as Artifacts
                       │
                       v
┌────────────────────────────────────┐
│  JOB 3: Create GitHub Release      │
│  (Only if version_changed=true)    │
└────────────────────────────────────┘
     │
     v
Download All Artifacts
     │
     v
Create Release v0.2.0
     │
     ├─> Release Notes
     ├─> Installation Instructions  
     ├─> Attach: reader-buddy-armv7-*.tar.gz
     └─> Attach: reader-buddy-aarch64-*.tar.gz
     │
     v
✅ Release Published!
```

## Commit Message Impact

```
┌──────────────────────────────────────────────────────────────────────┐
│                    COMMIT → VERSION MAPPING                           │
└──────────────────────────────────────────────────────────────────────┘

Commit Type              Version Bump              Example
────────────────────────────────────────────────────────────────────────
feat: new feature        Minor (0.1.0 → 0.2.0)    feat: add page nav
fix: bug fix             Patch (0.1.0 → 0.1.1)    fix: keyboard timing
BREAKING CHANGE          Major (0.1.0 → 1.0.0)    feat: redesign API
docs: documentation      None                     docs: update README
chore: maintenance       None                     chore: update deps
style: formatting        None                     style: fix spacing
refactor: code reorg     None                     refactor: extract fn
test: add tests          None                     test: add unit tests
perf: performance        Patch (0.1.0 → 0.1.1)    perf: optimize algo
────────────────────────────────────────────────────────────────────────
```

## Multi-Commit Scenarios

```
Scenario 1: Mixed Commits
───────────────────────────────────────────────────────
Commits:
  • docs: update README
  • fix: resolve bug
  • feat: add feature
  • chore: update deps

Result: 0.1.0 → 0.2.0 (highest = feat = minor bump)


Scenario 2: Fixes Only
───────────────────────────────────────────────────────
Commits:
  • fix: bug 1
  • fix: bug 2
  • docs: update

Result: 0.1.0 → 0.1.1 (highest = fix = patch bump)


Scenario 3: No Version Bumps
───────────────────────────────────────────────────────
Commits:
  • docs: improve docs
  • chore: update CI
  • style: format code

Result: 0.1.0 (no change, no release created)


Scenario 4: Breaking Change
───────────────────────────────────────────────────────
Commits:
  • feat: add feature
  • fix: small bug
  • feat: redesign API
    
    BREAKING CHANGE: API restructured

Result: 0.1.0 → 1.0.0 (BREAKING CHANGE = major bump)
```

## CI Workflow Detail

```
┌─────────────────────────────────────────────────────────────────────────┐
│                            CI WORKFLOW                                   │
│                   (Runs on PRs and Pushes)                               │
└─────────────────────────────────────────────────────────────────────────┘

PR Created/Updated
     │
     v
┌────────────────────────────────────┐
│  JOB: Check & Lint                 │
└────────────────────────────────────┘
     │
     ├─> cargo fmt --check
     │   └─> ✅ Formatted correctly
     │        or ❌ Formatting issues found
     │
     ├─> cargo clippy --all-targets
     │   └─> ✅ No linting warnings
     │        or ❌ Clippy warnings found
     │
     ├─> cargo check --target=armv7-unknown-linux-gnueabihf
     │   └─> ✅ Compiles for reMarkable 2
     │        or ❌ Compilation failed
     │
     ├─> cargo check --target=aarch64-unknown-linux-gnu
     │   └─> ✅ Compiles for reMarkable Paper Pro
     │        or ❌ Compilation failed
     │
     v
┌────────────────────────────────────┐
│  JOB: Test                         │
└────────────────────────────────────┘
     │
     ├─> cargo test --all-features
     │   └─> ✅ All tests passed
     │        or ❌ Tests failed
     │
     v
✅ CI Complete - Ready for Merge
or
❌ CI Failed - Fix Issues Before Merge
```

## User Experience Flow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                        END USER WORKFLOW                                 │
└─────────────────────────────────────────────────────────────────────────┘

User Wants Reader Buddy
     │
     v
Navigate to GitHub Releases
     │
     v
Download Latest Release
     │
     ├─> reader-buddy-armv7-*.tar.gz (for RM2)
     └─> reader-buddy-aarch64-*.tar.gz (for RMPP)
     │
     v
Extract Binary
     │
     v
Copy to reMarkable
     │
     v
Set OPENAI_API_KEY
     │
     v
Run ./reader-buddy
     │
     v
✅ Using Reader Buddy!
```

## File Changes Through Workflow

```
┌─────────────────────────────────────────────────────────────────────────┐
│                    FILE CHANGES IN RELEASE                               │
└─────────────────────────────────────────────────────────────────────────┘

Before Release               After MagDrago Action          After Build
─────────────────────────────────────────────────────────────────────────
Cargo.toml                   Cargo.toml                     + Binaries
version = "0.1.0"            version = "0.2.0"              + Release
                             ↓                              
                             Committed to repo              
                             ↓                              
                             Git tag: v0.2.0                
                             ↓                              
                             GitHub Release: v0.2.0         
                                                            with artifacts
```

## Summary Diagram

```
                             ┌──────────────┐
                             │  Developer   │
                             └──────┬───────┘
                                    │
                           Commits with conventional format
                                    │
                                    v
                      ┌─────────────────────────────┐
                      │      GitHub Actions         │
                      └─────────────────────────────┘
                                    │
                    ┌───────────────┴───────────────┐
                    │                               │
                    v                               v
           ┌────────────────┐            ┌──────────────────┐
           │   CI Workflow  │            │ Release Workflow │
           │   • Format     │            │  • MagDrago      │
           │   • Lint       │            │  • Build         │
           │   • Check      │            │  • Release       │
           │   • Test       │            └──────────────────┘
           └────────────────┘                     │
                                                  v
                                    ┌──────────────────────────┐
                                    │   GitHub Release         │
                                    │   • Binaries attached    │
                                    │   • Version tagged       │
                                    │   • Notes included       │
                                    └──────────────────────────┘
                                                  │
                                                  v
                                          ┌───────────────┐
                                          │   End Users   │
                                          │  Download &   │
                                          │     Use       │
                                          └───────────────┘
```

---

**Note**: All diagrams are text-based for easy viewing in any environment.

