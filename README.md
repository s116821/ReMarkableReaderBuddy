# ReMarkable Reader Buddy

An AI-powered reading assistant for the reMarkable tablet that watches for circled content and handwritten questions, then provides answers directly on your device using ChatGPT.

## Features

- **Content Outline Detection**: Automatically detects content you've outlined on your reMarkable (circles, rectangles, or any closed shape)
- **Question Extraction**: Uses vision AI to read your handwritten question near the outline
- **Intelligent Answers**: Queries ChatGPT with the outlined content and your question
- **New Page Creation**: Adds a blank page after the current one to display the answer
- **On-Device Rendering**: Displays question and answer directly on your reMarkable tablet
- **Smart Cleanup**: Erases original question and marks it with a reference symbol
- **Symbol Linking**: Places matching reference symbols on both the original page and answer page

## How It Works

1. **Outline Content**: Draw any closed shape (circle, rectangle, etc.) around content you want to ask about
2. **Write Question**: Write your question near the outlined content
3. **Trigger**: Touch the **lower-right corner** of your reMarkable screen with your hand
4. **Capture**: The app takes a screenshot of your current page
5. **AI Magic**: Single ChatGPT vision call detects outline, reads question, and generates answer (all in one!)
6. **Create Page**: Inserts a new blank page after the current one
7. **Render**: Displays the question and answer on the new page
8. **Mark & Link**: Erases the original question (but leaves the outline intact), places a reference symbol (①, ②, ③, etc.) on both pages to link them

**Note**: Processes one outline-question pair per trigger. Future versions may support multiple pairs in a single session.

## Installation

### Prerequisites

- reMarkable 2 or reMarkable Paper Pro in developer mode
- SSH access to your reMarkable
- OpenAI API key
- Rust toolchain and `cross` for cross-compilation

### Building

```bash
# Install cross-compilation tool
cargo install cross --git https://github.com/cross-rs/cross

# Add targets
rustup target add armv7-unknown-linux-gnueabihf aarch64-unknown-linux-gnu

# Build for reMarkable2
./build.sh rm2

# Or build for reMarkable Paper Pro
./build.sh rmpp
```

### Deploying

#### Option 1: Download Pre-built Binary (Recommended)

Download the latest release from the [Releases page](https://github.com/s116821/ReMarkableReaderBuddy/releases):

```bash
# Extract the binary
tar xzf reader-buddy-armv7-unknown-linux-gnueabihf.tar.gz  # For reMarkable 2
# or
tar xzf reader-buddy-aarch64-unknown-linux-gnu.tar.gz      # For Paper Pro

# Copy to reMarkable (replace IP address)
scp reader-buddy root@10.11.99.1:

# SSH into reMarkable
ssh root@10.11.99.1

# Set environment variables
export OPENAI_API_KEY=your-key-here

# Run the application
./reader-buddy
```

#### Option 2: Build from Source

```bash
# Build using the script
./build.sh rm2    # or ./build.sh rmpp

# Copy to reMarkable (replace IP address)
scp target/armv7-unknown-linux-gnueabihf/release/reader-buddy root@10.11.99.1:

# SSH into reMarkable
ssh root@10.11.99.1

# Set environment variables
export OPENAI_API_KEY=your-key-here

# Run the application
./reader-buddy
```

## Configuration

### Environment Variables

- `OPENAI_API_KEY`: Your OpenAI API key (required)
- `OPENAI_BASE_URL`: Custom API endpoint (optional)

### Command Line Options

```bash
reader-buddy [OPTIONS]

Options:
  --api-key <KEY>           OpenAI API key
  --model <MODEL>           Model to use [default: gpt-4o]
  --base-url <URL>          Custom OpenAI endpoint
  --no-draw                 Disable drawing (testing)
  --no-trigger              Skip waiting for trigger
  --once                    Run once instead of looping
  --input-png <FILE>        Use image file instead of screenshot
  --save-screenshot <FILE>  Save screenshot to file
  --trigger-corner <CORNER> Trigger corner: UR, UL, LR, LL [default: LR]
  --log-level <LEVEL>       Log level [default: info]
  -h, --help                Print help
  -V, --version             Print version
```

## Usage Examples

### Basic Usage

```bash
# Run with default settings (requires OPENAI_API_KEY env var)
./reader-buddy

# Run with explicit API key
./reader-buddy --api-key sk-...

# Use different model
./reader-buddy --model gpt-4o-mini

# Change trigger corner to upper-right (default is lower-right)
./reader-buddy --trigger-corner UR
```

### Testing

```bash
# Test with a sample image
./reader-buddy --input-png test.png --no-trigger --once --save-screenshot output.png

# Run without drawing to screen (logs only)
./reader-buddy --no-draw --once
```

### Background Execution

```bash
# Run in background
nohup ./reader-buddy > reader-buddy.log 2>&1 &

# Check logs
tail -f reader-buddy.log

# Stop background process
pkill reader-buddy
```

## Development

### Development Setup

```bash
# Install Rust toolchain (if not already installed)
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh

# Install formatting and linting tools (usually included with Rust)
rustup component add rustfmt clippy

# Install cross-compilation tool
cargo install cross --git https://github.com/cross-rs/cross

# Add target architectures for reMarkable devices
rustup target add armv7-unknown-linux-gnueabihf    # reMarkable 2
rustup target add aarch64-unknown-linux-gnu         # reMarkable Paper Pro
```

### Common Development Tasks

```bash
# Format code
cargo fmt

# Check formatting without making changes
cargo fmt -- --check

# Run linter
cargo clippy

# Run clippy with strict warnings
cargo clippy -- -D warnings

# Check that code compiles
cargo check --all-targets --all-features

# Build for reMarkable
./build.sh rm2    # or rmpp for Paper Pro
```

**Architecture**: Modular design with device, llm, analysis, and workflow layers. Device interaction code adapted from [awwaiid/ghostwriter](https://github.com/awwaiid/ghostwriter).

**Technical Details**: See [docs/TECHNICAL.md](docs/TECHNICAL.md) for complete architecture documentation and implementation notes.

## Troubleshooting

### "No xochitl process found"
Make sure your reMarkable is not in sleep mode and has a document open.

### "OPENAI_API_KEY not set"
Set the environment variable: `export OPENAI_API_KEY=your-key`

### "No outlined regions found"
- Make sure you've drawn a closed shape around content (circle, rectangle, or any outline)
- Write your question near the outlined area
- Try using darker/clearer pen strokes
- Ensure the outline is complete (no gaps)

### Touch trigger not working
- Verify the trigger corner setting (default is **lower-right**)
- Make sure you're using your hand/finger, not the pen
- The trigger zone is 68x68 pixels in the specified corner
- Try touching and holding for a moment before releasing

## Known Limitations

- **Single Question Per Trigger**: Processes one outline-question pair per trigger (future: may support multiple if use case emerges)
- **Outline Detection**: Currently LLM-based (future: add local CV algorithms as optimization)
- **Symbol Rendering**: Currently simple geometric shapes (TODO: render actual ①②③④⑤ glyphs)
- **Bounding Box Accuracy**: LLM provides approximate regions (may need fine-tuning for precise erasure)
- **Internet Required**: Requires connection for ChatGPT API
- **No Context Retention**: Each trigger is independent (no follow-up question support in v0.1)

## Automated Releases

This project uses **[MagDrago Rust Semver Action](https://github.com/s116821/MagDragoRustSemverAction)** for automated versioning and releases.

**Version Bump Rules**:
- **Major** (X.0.0): Commit with `!` (e.g., `feat!: breaking change`)
- **Minor** (0.X.0): Merge from `feature/` branch
- **Patch** (0.0.X): Any source file change
- **None**: Docs-only changes

Releases are automatically created with pre-built binaries for both reMarkable devices.

## Contributing

Contributions welcome! Remaining enhancements:
- Proper ①②③④⑤ glyph rendering (currently simple shapes)
- Local CV outline detection (reduce LLM calls)
- Improved bounding box accuracy
- Device testing and refinement

**Version Bumps**: Use `!` for major, `feature/` branches for minor, any code change for patch.

See [docs/TECHNICAL.md](docs/TECHNICAL.md) for implementation details and TODOs.

## License

See LICENSE file for details.

## Documentation

- **[Technical Documentation](docs/TECHNICAL.md)** - Architecture, implementation details, and TODOs
- **[Workflow Diagrams](docs/WORKFLOW_DIAGRAM.md)** - Visual CI/CD and app workflow diagrams

## Acknowledgments

- [awwaiid/ghostwriter](https://github.com/awwaiid/ghostwriter) - Core device interaction code
- [MagDrago Rust Semver Action](https://github.com/s116821/MagDragoRustSemverAction) - Automated versioning
- reMarkable community for documentation and tools
- OpenAI for GPT vision capabilities
