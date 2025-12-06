# ReMarkable Reader Buddy

An AI-powered reading assistant for the reMarkable tablet that watches for circled content and handwritten questions, then provides answers directly on your device using ChatGPT.

## Features

- **Content Outline Detection**: Automatically detects content you've outlined on your reMarkable (circles, rectangles, or any closed shape)
- **Question Extraction**: Uses vision AI to read your handwritten question near the outline
- **Intelligent Answers**: Queries ChatGPT with the outlined content and your question
- **On-Device Rendering**: Displays question and answer directly on your reMarkable tablet
- **Answer Page Detection**: Recognizes blank pages or existing answer pages for seamless Q&A flow

## How It Works

1. **Prepare Answer Page**: Before triggering, create a blank page to the **right** of your question page
2. **Outline Content**: Draw any closed shape (circle, rectangle, etc.) around content you want to ask about
3. **Write Question**: Write your question near the outlined content
4. **Trigger**: Touch the **lower-right corner** of your reMarkable screen with your hand
5. **Capture**: The app takes a screenshot of your current page
6. **AI Magic**: Single ChatGPT vision call detects outline, reads question, and generates answer (all in one!)
7. **Page Check**: App navigates right and checks for a valid answer page:
   - **Valid**: Blank page or existing Reader Buddy answer page → renders Q&A
   - **Invalid**: No page exists or page has other content → draws an **X** in the bottom-right corner of the original page
8. **Render**: Displays the question and answer on the answer page (with "=== Reader Buddy Answers ===" header on first use)

**Important**: You must manually create a blank page to the right of your question page before triggering. The app will NOT create pages automatically.

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
  --debug-dump              Save debug images to /tmp for troubleshooting
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

### Run at Boot (systemd)

To have Reader Buddy start automatically when your reMarkable boots:

**Prerequisites:** Ensure the binary is copied to `/home/root/reader-buddy` on your reMarkable:

```bash
# From your computer, copy the binary to the reMarkable
scp reader-buddy root@10.11.99.1:/home/root/reader-buddy

# Or if building from source:
scp target/armv7-unknown-linux-gnueabihf/release/reader-buddy root@10.11.99.1:/home/root/reader-buddy
# For Paper Pro use: target/aarch64-unknown-linux-gnu/release/reader-buddy
```

**1. Create the systemd service file:**

```bash
ssh root@10.11.99.1
cat > /etc/systemd/system/reader-buddy.service << 'EOF'
[Unit]
Description=ReMarkable Reader Buddy
After=home.mount xochitl.service
Wants=xochitl.service

[Service]
Type=simple
Environment="OPENAI_API_KEY=your-api-key-here"
ExecStart=/home/root/reader-buddy
Restart=on-failure
RestartSec=5
StandardOutput=journal
StandardError=journal

[Install]
WantedBy=multi-user.target
EOF
```

> **Important**: Replace `your-api-key-here` with your actual OpenAI API key (starts with `sk-`).

**2. Enable and start the service:**

```bash
# Reload systemd to recognize the new service
systemctl daemon-reload

# Enable the service to start at boot
systemctl enable reader-buddy.service

# Start the service now
systemctl start reader-buddy.service

# Check service status
systemctl status reader-buddy.service
```

**3. Managing the service:**

```bash
# Stop the service
systemctl stop reader-buddy.service

# Restart the service
systemctl restart reader-buddy.service

# Disable auto-start at boot
systemctl disable reader-buddy.service

# Remove the service completely
systemctl stop reader-buddy.service
systemctl disable reader-buddy.service
rm /etc/systemd/system/reader-buddy.service
systemctl daemon-reload
```

### Viewing Logs with journalctl

When running as a systemd service, logs are captured by the journal system:

```bash
# View all Reader Buddy logs
journalctl -u reader-buddy.service

# Follow logs in real-time (like tail -f)
journalctl -u reader-buddy.service -f

# View logs since last boot
journalctl -u reader-buddy.service -b

# View last 100 lines
journalctl -u reader-buddy.service -n 100

# View logs from the last hour
journalctl -u reader-buddy.service --since "1 hour ago"

# View logs with timestamps
journalctl -u reader-buddy.service -o short-precise

# View only error-level logs
journalctl -u reader-buddy.service -p err
```

**Common debugging scenarios:**

```bash
# Check why the service failed to start
journalctl -u reader-buddy.service -b --no-pager

# Watch logs while testing (in one SSH session)
journalctl -u reader-buddy.service -f

# Then trigger Reader Buddy from your tablet and watch the output
```

**Tip**: If logs aren't appearing, ensure `StandardOutput=journal` and `StandardError=journal` are set in the service file. You can also add `--log-level debug` to the `ExecStart` line for more verbose output:

```bash
ExecStart=/home/root/reader-buddy --log-level debug
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

### X appears on my question page instead of answer
If you see an X drawn in the bottom-right corner of your question page, it means the app could not find a valid answer page. This happens when:
- **No page to the right**: You need to manually create a blank page to the right of your question page before triggering
- **Page has existing content**: The page to the right has content that isn't a Reader Buddy answer page (e.g., your notes, a different document page)

**Solution**: Navigate to the page with your question, add a new blank page to the right using the reMarkable's page menu, then trigger Reader Buddy again.

### Answer not appearing on new page
If the answer doesn't render and no X appears:
- Enable debug logging: `--log-level debug` to see detailed execution flow
- Check that the answer page was detected correctly (logs will show "Valid answer page found")

### Debug Mode
Enable debug dumps to troubleshoot rendering issues:
```bash
./reader-buddy --debug-dump --log-level debug
```

This will save to `/tmp/` on the reMarkable:
- `reader-buddy-screenshot-*.png` - Original screenshots captured
- `reader-buddy-erase-mask-*.png` - Visual overlay showing detected question regions (red box) and ink pixels to erase (yellow)

**Copying debug files to your computer:**

On Windows (PowerShell):
```powershell
# Copy all debug images from reMarkable to current directory
scp root@10.11.99.1:/tmp/reader-buddy-*.png .

# Or copy to a specific folder
scp root@10.11.99.1:/tmp/reader-buddy-*.png C:\path\to\debug\folder\
```

On Linux/Mac:
```bash
# Copy all debug images from reMarkable to current directory
scp root@10.11.99.1:/tmp/reader-buddy-*.png .

# Or copy to a specific folder
scp root@10.11.99.1:/tmp/reader-buddy-*.png ~/debug/
```

Replace `10.11.99.1` with your reMarkable's IP address. You can find the IP address in **Settings > Help > Copyrights and licenses** at the bottom.

## Cleanup and Uninstall

### Removing Reader Buddy from your reMarkable

SSH into your reMarkable and remove the binary:
```bash
ssh root@10.11.99.1
rm -f ~/reader-buddy
```

### Cleaning up debug files

Remove debug images from the reMarkable:
```bash
ssh root@10.11.99.1
rm -f /tmp/reader-buddy-*.png
```

### Removing persistent state

Reader Buddy stores a header pattern to recognize existing answer pages. To reset this:
```bash
ssh root@10.11.99.1
rm -f /home/root/.reader-buddy-header-pattern.png
```

### Complete cleanup (all at once)

```bash
ssh root@10.11.99.1 "rm -f ~/reader-buddy /tmp/reader-buddy-*.png /home/root/.reader-buddy-header-pattern.png"
```

### Stopping a running instance

If Reader Buddy is running in the background:
```bash
ssh root@10.11.99.1
# Find the process
ps | grep reader-buddy

# Kill it (replace PID with actual process ID from the output)
kill <PID>

# Or kill using killall (kills all instances)
killall reader-buddy
```

## Known Limitations

- **Manual Page Creation Required**: You must create a blank page to the right of your question page before triggering - the app does not create pages automatically
- **Single Question Per Trigger**: Processes one outline-question pair per trigger
- **Outline Detection**: Currently LLM-based (future: add local CV algorithms as optimization)
- **Internet Required**: Requires connection for ChatGPT API
- **No Context Retention**: Each trigger is independent (no follow-up question support)

## Automated Releases

This project uses **[MagDrago Rust Semver Action](https://github.com/s116821/MagDragoRustSemverAction)** for automated versioning and releases.

**Version Bump Rules**:
- **Major** (X.0.0): Commit with `!` (e.g., `feat!: breaking change`)
- **Minor** (0.X.0): Merge from `feature/` branch
- **Patch** (0.0.X): Any source file change
- **None**: Docs-only changes

Releases are automatically created with pre-built binaries for both reMarkable devices.

## Contributing

Contributions welcome! Areas for enhancement:
- Local CV outline detection (reduce LLM calls)
- Multi-question support per trigger
- Device testing and refinement
- Automatic page creation (currently requires manual page setup)

**Recent Improvements (v0.3)**:
- ✅ Simplified workflow - user creates answer page, app detects blank/QA pages
- ✅ Clear failure indication - X drawn in bottom-right when no valid answer page found
- ✅ Debug dump mode for troubleshooting
- ✅ Answer page reuse - multiple questions from same page share one answer page

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
