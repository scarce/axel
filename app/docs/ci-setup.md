# CI/CD Setup for Axel

This document describes the CI/CD pipeline for building and distributing Axel.

## GitHub Actions Workflow

The workflow is defined in `.github/workflows/build.yml` and:

1. **Triggers on:**
   - Git tags matching `v*` (e.g., `v1.0.0`)
   - Manual workflow dispatch with optional version input

2. **Build Steps:**
   - Checks out code with submodules
   - Selects Xcode 16.2
   - Installs `create-dmg` via Homebrew
   - Builds the macOS Release app
   - Creates a DMG with drag-and-drop installer
   - Optionally signs, notarizes, and staples the DMG
   - Uploads artifact and creates GitHub Release

## Required Secrets

### Code Signing and Notarization (optional)

| Secret | Description |
|--------|-------------|
| `MACOS_CERTIFICATE` | Base64-encoded Developer ID certificate (.p12) |
| `MACOS_CERTIFICATE_PWD` | Password for the .p12 certificate |
| `KEYCHAIN_PASSWORD` | Temporary keychain password (can be any value) |
| `APPLE_ID` | Apple ID email for notarization |
| `APPLE_PASSWORD` | App-specific password for notarization |
| `APPLE_TEAM_ID` | Apple Developer Team ID (e.g., `8ZJ55A62XN`) |

### S3 Publishing (required for releases)

| Secret | Description |
|--------|-------------|
| `AWS_KEY_ID` | AWS Access Key ID with S3 write permissions |
| `AWS_SECRET_ACCESS_KEY` | AWS Secret Access Key |

The DMG is published to:
- **Versioned:** `s3://txtx-public/releases/axel/Axel-{version}-macos.dmg`
- **Latest:** `s3://txtx-public/releases/axel/Axel-latest-macos.dmg`

Public URLs:
- `https://txtx-public.s3.amazonaws.com/releases/axel/Axel-{version}-macos.dmg`
- `https://txtx-public.s3.amazonaws.com/releases/axel/Axel-latest-macos.dmg`

### Setting Up Secrets

1. **Export your Developer ID certificate:**
   ```bash
   # In Keychain Access, export "Developer ID Application" certificate as .p12
   # Then encode it:
   base64 -i certificate.p12 | pbcopy
   ```

2. **Create app-specific password:**
   - Go to https://appleid.apple.com/account/manage
   - Under "Sign-In and Security" → "App-Specific Passwords"
   - Generate a new password for "Axel CI"

3. **Add secrets to GitHub:**
   - Go to Repository → Settings → Secrets and variables → Actions
   - Add each secret listed above

## Local Testing

### Test DMG Creation

```bash
# Install create-dmg
brew install create-dmg

# Build the app
xcodebuild -scheme Axel -project Axel.xcodeproj -configuration Release \
  -destination "platform=macOS" -derivedDataPath build build

# Create DMG
./scripts/create-dmg.sh "build/Build/Products/Release/Axel.app" "Axel.dmg"
```

### Regenerate DMG Background

The DMG background images are in `assets/`:
- `dmg-background.png` (1x)
- `dmg-background@2x.png` (2x for Retina)

To regenerate with a nicer design (requires Pillow):
```bash
pip3 install Pillow
python3 scripts/generate-dmg-background.py
```

Or use the simple gradient version (no dependencies):
```bash
python3 scripts/generate-dmg-background-simple.py
```

## Creating a Release

1. **Tag the commit:**
   ```bash
   git tag v1.0.0
   git push origin v1.0.0
   ```

2. **The workflow will automatically:**
   - Build the macOS app
   - Create a signed DMG (if certificates are configured)
   - Notarize with Apple (if credentials are configured)
   - Create a draft GitHub Release

3. **Finalize the release:**
   - Go to GitHub Releases
   - Edit the draft release
   - Add release notes
   - Publish

## DMG Layout

The DMG installer uses a dark gradient background matching Axel's theme:

```
┌────────────────────────────────────────────────┐
│                                                │
│                                                │
│      [Axel.app]  ────────→  [Applications]     │
│                                                │
│                                                │
│        Drag Axel to Applications to install    │
└────────────────────────────────────────────────┘
```

Window size: 600×400
Icon positions: Axel at (150, 190), Applications at (450, 190)
