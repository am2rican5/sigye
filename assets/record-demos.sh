#!/bin/bash
# Record all demo GIFs for sigye
#
# Prerequisites:
#   brew install vhs
#   cargo build --release
#
# Usage:
#   cd assets && ./record-demos.sh               # Record all demos
#   cd assets && ./record-demos.sh main          # Record only the main demo
#   cd assets && ./record-demos.sh quick         # Record only the quick demo
#   cd assets && ./record-demos.sh screensaver   # Record only the screensaver demo

set -e

SCRIPT_DIR="$(cd "$(dirname "$0")" && pwd)"
cd "$SCRIPT_DIR/.."

# Check dependencies
if ! command -v vhs &> /dev/null; then
    echo "Error: vhs not found. Install with: brew install vhs"
    exit 1
fi

# Build release binary
echo "Building sigye (release)..."
cargo build --release

cd assets

record() {
    local tape="$1"
    local name="${tape%.tape}"
    echo ""
    echo "Recording $tape → ${name}.gif ..."
    vhs "$tape"
    echo "Done: ${name}.gif ($(du -h "${name}.gif" | cut -f1))"
}

case "${1:-all}" in
    main)
        record demo.tape
        ;;
    quick)
        record demo-quick.tape
        ;;
    screensaver)
        record demo-screensaver.tape
        ;;
    all)
        record demo.tape
        record demo-quick.tape
        record demo-screensaver.tape
        ;;
    *)
        echo "Usage: $0 [main|quick|screensaver|all]"
        exit 1
        ;;
esac

echo ""
echo "All done! GIFs are in the assets/ directory."
echo ""
echo "Tips for optimal GIF size:"
echo "  - Use gifsicle to optimize: gifsicle -O3 demo.gif -o demo-optimized.gif"
echo "  - Or convert to mp4 for smaller size: ffmpeg -i demo.gif demo.mp4"
