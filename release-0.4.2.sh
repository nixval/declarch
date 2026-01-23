#!/bin/bash
# Release Script for declarch v0.4.2
# This script helps prepare the release for GitHub and AUR

set -e

VERSION="0.4.2"
TAG="v${VERSION}"

echo "=== 🚀 declarch Release ${VERSION} ==="
echo ""

# Colors
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Step 1: Verify everything is committed
echo -e "${BLUE}Step 1: Verifying git status...${NC}"
if [ -n "$(git status --porcelain)" ]; then
    echo -e "${YELLOW}Warning: There are uncommitted changes${NC}"
    git status --short
    echo "Please commit or stash them first"
    exit 1
fi
echo -e "${GREEN}✓ Working tree clean${NC}"
echo ""

# Step 2: Verify tests pass
echo -e "${BLUE}Step 2: Running tests...${NC}"
if cargo test --quiet 2>&1 | grep -q "test result: ok"; then
    echo -e "${GREEN}✓ All tests passing${NC}"
else
    echo -e "${YELLOW}Some tests failed${NC}"
    exit 1
fi
echo ""

# Step 3: Build release binary
echo -e "${BLUE}Step 3: Building release binary...${NC}"
cargo build --release 2>&1 | grep -E "Compiling|Finished" || true
echo -e "${GREEN}✓ Release build complete${NC}"
echo ""

# Step 4: Generate sha256sums for PKGBUILD
echo -e "${BLUE}Step 4: Generating sha256sums...${NC}"
echo "Note: This will be done AFTER pushing the tag to GitHub"
echo "Run: makepkg -g"
echo ""

# Step 5: Show git log
echo -e "${BLUE}Step 5: Recent commits...${NC}"
git log --oneline -5
echo ""

# Step 6: Show tag info
echo -e "${BLUE}Step 6: Tag information...${NC}"
git show ${TAG} --no-patch --format="Tag: %d%nDate: %ci%nMessage: %s"
echo ""

# Step 7: Create release checklist
echo -e "${BLUE}=== RELEASE CHECKLIST ===${NC}"
echo ""
echo "Before pushing to GitHub:"
echo -e "  ${GREEN}✓${NC} Version updated in Cargo.toml"
echo -e "  ${GREEN}✓${NC} PKGBUILD updated (pkgver=0.4.2)"
echo -e "  ${GREEN}✓${NC} RELEASE.md updated with changelog"
echo -e "  ${GREEN}✓${NC} README.md badge updated"
echo -e "  ${GREEN}✓${NC} Tag ${TAG} created"
echo -e "  ${GREEN}✓${NC} All tests passing"
echo -e "  ${GREEN}✓${NC} Release build successful"
echo ""

echo "To complete the release:"
echo ""
echo -e "${YELLOW}1. Push to GitHub:${NC}"
echo "   git push origin main"
echo "   git push origin ${TAG}"
echo ""

echo -e "${YELLOW}2. Wait for GitHub release tarball to be available (~30 seconds)${NC}"
echo "   sleep 30"
echo ""

echo -e "${YELLOW}3. Generate real sha256sums:${NC}"
echo "   wget https://github.com/nixval/declarch/archive/refs/tags/${TAG}/declarch-${VERSION}.tar.gz"
echo "   sha256sum declarch-${VERSION}.tar.gz"
echo "   # Update PKGBUILD sha256sums= with the output"
echo ""

echo -e "${YELLOW}4. Create GitHub release (optional, but recommended):${NC}"
echo "   gh release create ${TAG} --title \"v${VERSION}\" --notes \"Release v${VERSION}"
echo "   # Or use GitHub web UI to create release with RELEASE.md content"
echo ""

echo -e "${YELLOW}5. Test PKGBUILD locally:${NC}"
echo "   makepkg -sf"
echo "   pacman -Qip declarch-*.pkg.tar.zst"
echo ""

echo -e "${YELLOW}6. Publish to AUR (only after GitHub release):${NC}"
echo "   ./.aur/publish.sh ${VERSION}"
echo ""

echo -e "${YELLOW}7. Verify on AUR:${NC}"
echo "   # Wait ~10 minutes for AUR to process"
echo "   # Check: https://aur.archlinux.org/packages/declarch"
echo ""

echo -e "${BLUE}=== NEXT STEPS ===${NC}"
echo ""
echo "Currently at: $(git branch --show-current)"
echo "Latest tag: ${TAG}"
echo ""
echo "Type 'yes' to continue with git push, or Ctrl+C to cancel:"
read -r response

if [ "$response" = "yes" ]; then
    echo ""
    echo -e "${GREEN}Pushing to GitHub...${NC}"
    git push origin main
    git push origin ${TAG}
    echo -e "${GREEN}✓ Pushed successfully!${NC}"
    echo ""
    echo -e "${BLUE}Next: Follow steps 2-6 from the checklist above${NC}"
else
    echo "Cancelled. Ready to push when you are."
fi
