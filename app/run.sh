#!/bin/bash
set -e

cd "$(dirname "$0")"

PLATFORM="${1:-macos}"
OS_VERSION="${2:-}"

# Prefer Xcode.app if available (allows using newer SDKs)
XCODE_APP="/Applications/Xcode.app/Contents/Developer"
if [[ -d "$XCODE_APP" ]]; then
  export DEVELOPER_DIR="$XCODE_APP"
fi

# Kill existing instance (macOS only)
if [[ "$PLATFORM" == "macos" ]]; then
  pkill -9 Axel 2>/dev/null || true
fi

# Build
echo "Building ($PLATFORM)..."
DERIVED_DATA_PATH="$(pwd)/build/DerivedData-${PLATFORM}"
PACKAGE_CACHE_PATH="$(pwd)/build/SourcePackages"
TMPDIR_PATH="$(pwd)/build/tmp"
HOME_PATH="$(pwd)/build/home"
mkdir -p "$DERIVED_DATA_PATH" "$PACKAGE_CACHE_PATH" "$TMPDIR_PATH" "$HOME_PATH"
export TMPDIR="$TMPDIR_PATH"
export HOME="$HOME_PATH"

DESTINATION="platform=macOS"
EXTRA_ARGS=()
SDK_ARGS=()
if [[ "$PLATFORM" == "ios" ]]; then
  DESTINATION="generic/platform=iOS"
  if [[ -n "$OS_VERSION" ]]; then
    SDK_ARGS+=("-sdk" "iphoneos${OS_VERSION}")
  fi
  EXTRA_ARGS+=(CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY="")
elif [[ "$PLATFORM" == "visionos" ]]; then
  DESTINATION="generic/platform=visionOS"
  if [[ -n "$OS_VERSION" ]]; then
    SDK_ARGS+=("-sdk" "xros${OS_VERSION}")
  fi
  EXTRA_ARGS+=(CODE_SIGNING_ALLOWED=NO CODE_SIGNING_REQUIRED=NO CODE_SIGN_IDENTITY="")
fi

xcodebuild \
  -project Axel.xcodeproj \
  -scheme Axel \
  -destination "$DESTINATION" \
  -configuration Debug \
  -derivedDataPath "$DERIVED_DATA_PATH" \
  -clonedSourcePackagesDirPath "$PACKAGE_CACHE_PATH" \
  "${SDK_ARGS[@]}" \
  "${EXTRA_ARGS[@]}" \
  build \
  -quiet

# Find and launch the built app (macOS only)
if [[ "$PLATFORM" == "macos" ]]; then
  APP_PATH=$(xcodebuild \
    -project Axel.xcodeproj \
    -scheme Axel \
    -destination 'platform=macOS' \
    -configuration Debug \
    -derivedDataPath "$DERIVED_DATA_PATH" \
    -showBuildSettings 2>/dev/null | grep -m1 "BUILT_PRODUCTS_DIR" | awk '{print $3}')
  echo "Launching..."
  open "$APP_PATH/Axel.app"
fi
