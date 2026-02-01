#!/usr/bin/env bash
#
# Build a release binary with embedded UI assets
#
# This script:
# 1. Builds the web UI using pnpm
# 2. Builds the Rust binary in release mode
# 3. Creates a zip of the UI assets
# 4. Appends the zip to the binary (zip files can be read from the end)
# 5. Outputs the final binary to dist/hyperlit
#

set -euo pipefail

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# 📖 # Why detect Windows and use appropriate null device?
# On Windows (MinGW/Git Bash), /dev/null doesn't exist as a special device.
# Redirecting to /dev/null creates a literal file named "nul" or "null" instead.
# We detect Windows environments and use the appropriate null device path.
NULL_DEV="/dev/null"
if [[ "$OSTYPE" == "msys" ]] || [[ "$OSTYPE" == "cygwin" ]] || [[ "$OSTYPE" == "win32" ]]; then
    # Windows environment - use Windows null device
    NULL_DEV="NUL"
fi

# Script directory and project root
SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
PROJECT_ROOT="$(cd "${SCRIPT_DIR}/.." && pwd)"

# Configuration
WEB_DIR="${PROJECT_ROOT}/web"
DIST_DIR="${PROJECT_ROOT}/dist"
RUST_BINARY="${PROJECT_ROOT}/target/release/hyperlit"
ASSETS_ZIP="${PROJECT_ROOT}/assets.zip"
FINAL_BINARY="${DIST_DIR}/hyperlit"

echo "🔨 Building hyperlit release binary with embedded UI assets"
echo "============================================================"

# Check prerequisites
check_prerequisites() {
    echo -e "${YELLOW}Checking prerequisites...${NC}"
    
    if ! command -v pnpm &> $NULL_DEV; then
        echo -e "${RED}Error: pnpm is not installed${NC}"
        echo "Please install pnpm: https://pnpm.io/installation"
        exit 1
    fi
    
    if ! command -v cargo &> $NULL_DEV; then
        echo -e "${RED}Error: cargo is not installed${NC}"
        echo "Please install Rust: https://rustup.rs/"
        exit 1
    fi
    
    if ! command -v zip &> $NULL_DEV; then
        echo -e "${RED}Error: zip command is not installed${NC}"
        echo "Please install zip (usually available via your package manager)"
        exit 1
    fi
    
    echo -e "${GREEN}✓ All prerequisites met${NC}"
}

# Build the web UI
build_web() {
    echo -e "${YELLOW}Building web UI...${NC}"
    
    if [[ ! -d "${WEB_DIR}" ]]; then
        echo -e "${RED}Error: Web directory not found at ${WEB_DIR}${NC}"
        exit 1
    fi
    
    cd "${WEB_DIR}"
    
    # Check if node_modules exists, if not install dependencies
    if [[ ! -d "node_modules" ]]; then
        echo "Installing npm dependencies..."
        pnpm install
    fi
    
    # Build the UI
    pnpm build
    
    # Verify build output
    if [[ ! -d "${WEB_DIR}/dist" ]]; then
        echo -e "${RED}Error: Build failed - dist directory not created${NC}"
        exit 1
    fi
    
    if [[ ! -f "${WEB_DIR}/dist/index.html" ]]; then
        echo -e "${RED}Error: Build failed - index.html not found in dist${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Web UI built successfully${NC}"
}

# Build the Rust binary
build_rust() {
    echo -e "${YELLOW}Building Rust binary...${NC}"
    
    cd "${PROJECT_ROOT}"
    
    # Build release binary
    cargo build --release --bin hyperlit
    
    # Verify binary exists
    if [[ ! -f "${RUST_BINARY}" ]]; then
        echo -e "${RED}Error: Rust binary not found at ${RUST_BINARY}${NC}"
        exit 1
    fi
    
    echo -e "${GREEN}✓ Rust binary built successfully${NC}"
}

# Create zip of UI assets
create_assets_zip() {
    echo -e "${YELLOW}Creating assets zip...${NC}"
    
    # Remove old zip if exists
    if [[ -f "${ASSETS_ZIP}" ]]; then
        rm "${ASSETS_ZIP}"
    fi
    
    # Create zip from web/dist
    cd "${WEB_DIR}/dist"
    zip -r "${ASSETS_ZIP}" . -q
    
    if [[ ! -f "${ASSETS_ZIP}" ]]; then
        echo -e "${RED}Error: Failed to create assets zip${NC}"
        exit 1
    fi
    
    local zip_size
    zip_size=$(du -h "${ASSETS_ZIP}" | cut -f1)
    echo -e "${GREEN}✓ Created assets.zip (${zip_size})${NC}"
}

# Append zip to binary
append_zip_to_binary() {
    echo -e "${YELLOW}Appending assets to binary...${NC}"
    
    # 📖 # Why create distribution binary first before appending?
    # On Windows, files that are in use cannot be overwritten.
    # If we appended to the original Rust binary and then tried to copy,
    # we might fail with "Device or resource busy" if the dist binary is running.
    # Instead, we copy first, then append, so the target binary is always fresh.
    
    # Create dist directory
    mkdir -p "${DIST_DIR}"
    
    # Copy binary to dist first
    cp "${RUST_BINARY}" "${FINAL_BINARY}"
    
    local original_size
    original_size=$(stat -f%z "${FINAL_BINARY}" 2>$NULL_DEV || stat -c%s "${FINAL_BINARY}" 2>$NULL_DEV)
    
    # Append the zip to the final binary
    cat "${ASSETS_ZIP}" >> "${FINAL_BINARY}"
    
    local new_size
    new_size=$(stat -f%z "${FINAL_BINARY}" 2>$NULL_DEV || stat -c%s "${FINAL_BINARY}" 2>$NULL_DEV)
    
    echo -e "${GREEN}✓ Assets embedded into binary${NC}"
    echo "  Original size: ${original_size} bytes"
    echo "  New size: ${new_size} bytes"
    echo "  Added: $((new_size - original_size)) bytes"
}

# Make binary executable on Unix systems
make_executable() {
    # 📖 # Why make the binary executable?
    # On Unix systems, copied binaries may not have the execute permission.
    # Windows doesn't require this, but it's harmless to run on Windows.
    if [[ "$OSTYPE" != "msys" ]] && [[ "$OSTYPE" != "cygwin" ]] && [[ "$OSTYPE" != "win32" ]]; then
        echo -e "${YELLOW}Making binary executable...${NC}"
        chmod +x "${FINAL_BINARY}"
        echo -e "${GREEN}✓ Binary is now executable${NC}"
    fi
}

# Cleanup temporary files
cleanup() {
    echo -e "${YELLOW}Cleaning up...${NC}"
    
    if [[ -f "${ASSETS_ZIP}" ]]; then
        rm "${ASSETS_ZIP}"
    fi
    
    echo -e "${GREEN}✓ Cleanup complete${NC}"
}

# Verify the build
verify_build() {
    echo -e "${YELLOW}Verifying build...${NC}"
    
    # Check if the binary contains a valid zip at the end
    if command -v unzip &> $NULL_DEV; then
        # Try to list contents of the embedded zip
        if unzip -l "${FINAL_BINARY}" > $NULL_DEV 2>&1; then
            echo -e "${GREEN}✓ Embedded assets are valid zip${NC}"
            echo ""
            echo "Zip contents preview:"
            unzip -l "${FINAL_BINARY}" | tail -15
        else
            echo -e "${YELLOW}Warning: Could not verify zip contents${NC}"
        fi
    fi
    
    local final_size
    final_size=$(du -h "${FINAL_BINARY}" | cut -f1)
    echo ""
    echo -e "${GREEN}✓ Build verification complete${NC}"
    echo "  Final binary size: ${final_size}"
}

# Main execution
main() {
    check_prerequisites
    echo ""
    build_web
    echo ""
    build_rust
    echo ""
    create_assets_zip
    echo ""
    append_zip_to_binary
    echo ""
    make_executable
    echo ""
    cleanup
    echo ""
    verify_build
    
    echo ""
    echo "============================================================"
    echo -e "${GREEN}✓ Release build complete!${NC}"
    echo ""
    echo "Binary location: ${FINAL_BINARY}"
    echo ""
    echo "Usage:"
    echo "  ./dist/hyperlit    # Run the binary"
    echo ""
    echo "The binary contains embedded UI assets and can be deployed"
    echo "as a single file. The UI will be served from the embedded zip."
}

# Trap to cleanup on error
trap cleanup EXIT

# Run main
main
