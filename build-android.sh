#!/bin/bash
set -e

echo "🤖 Building Clippy Share for Android"
echo "━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━"

# Colors
GREEN='\033[0;32m'
BLUE='\033[0;34m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Set Java 17 for this build
echo -e "${BLUE}☕ Setting Java 17...${NC}"
if [ -d "/usr/lib/jvm/java-17-openjdk" ]; then
    export JAVA_HOME=/usr/lib/jvm/java-17-openjdk
    export PATH=$JAVA_HOME/bin:$PATH
    echo -e "${GREEN}✅ Using Java 17: $(java -version 2>&1 | head -n 1)${NC}"
elif [ -d "/usr/lib/jvm/jdk-17" ]; then
    export JAVA_HOME=/usr/lib/jvm/jdk-17
    export PATH=$JAVA_HOME/bin:$PATH
    echo -e "${GREEN}✅ Using Java 17: $(java -version 2>&1 | head -n 1)${NC}"
else
    echo -e "${YELLOW}⚠️  Java 17 not found at standard locations${NC}"
    echo "Current Java: $(java -version 2>&1 | head -n 1)"
    echo "If build fails, manually set JAVA_HOME to Java 17"
fi
echo ""

# Check if NDK is installed
if [ -z "$ANDROID_NDK_HOME" ]; then
    echo "❌ ANDROID_NDK_HOME not set!"
    echo "Please install Android NDK and set ANDROID_NDK_HOME"
    echo "Example: export ANDROID_NDK_HOME=\$HOME/Android/Sdk/ndk/26.1.10909125"
    exit 1
fi

echo -e "${BLUE}📦 Installing Rust Android targets...${NC}"
rustup target add aarch64-linux-android
rustup target add armv7-linux-androideabi
rustup target add x86_64-linux-android

# Create cargo config for Android
mkdir -p .cargo
cat > .cargo/config.toml << EOF
[target.aarch64-linux-android]
ar = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/aarch64-linux-android30-clang"

[target.armv7-linux-androideabi]
ar = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/armv7a-linux-androideabi30-clang"

[target.x86_64-linux-android]
ar = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/llvm-ar"
linker = "$ANDROID_NDK_HOME/toolchains/llvm/prebuilt/linux-x86_64/bin/x86_64-linux-android30-clang"
EOF

# Build for each architecture
echo -e "${BLUE}🔨 Building for arm64-v8a...${NC}"
cargo build --release --target aarch64-linux-android -p mobile-bridge

echo -e "${BLUE}🔨 Building for armeabi-v7a...${NC}"
cargo build --release --target armv7-linux-androideabi -p mobile-bridge

echo -e "${BLUE}🔨 Building for x86_64...${NC}"
cargo build --release --target x86_64-linux-android -p mobile-bridge

# Create jniLibs directory structure
echo -e "${BLUE}📂 Copying libraries to jniLibs...${NC}"
mkdir -p android/app/src/main/jniLibs/arm64-v8a
mkdir -p android/app/src/main/jniLibs/armeabi-v7a
mkdir -p android/app/src/main/jniLibs/x86_64

# Copy the .so files
cp target/aarch64-linux-android/release/libmobile_bridge.so android/app/src/main/jniLibs/arm64-v8a/
cp target/armv7-linux-androideabi/release/libmobile_bridge.so android/app/src/main/jniLibs/armeabi-v7a/
cp target/x86_64-linux-android/release/libmobile_bridge.so android/app/src/main/jniLibs/x86_64/

echo -e "${GREEN}✅ Native libraries built successfully!${NC}"
echo ""
echo "Library locations:"
echo "  • arm64-v8a: android/app/src/main/jniLibs/arm64-v8a/libmobile_bridge.so"
echo "  • armeabi-v7a: android/app/src/main/jniLibs/armeabi-v7a/libmobile_bridge.so"
echo "  • x86_64: android/app/src/main/jniLibs/x86_64/libmobile_bridge.so"
echo ""

# Build APK
echo -e "${BLUE}📱 Building Android APK...${NC}"
cd android
./gradlew assembleDebug

echo ""
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo -e "${GREEN}✨ Build complete!${NC}"
echo -e "${GREEN}━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━━${NC}"
echo ""
echo "📦 APK location: android/app/build/outputs/apk/debug/app-debug.apk"
echo ""
echo "To install on device:"
echo "  adb install android/app/build/outputs/apk/debug/app-debug.apk"
echo ""
echo "To build release APK:"
echo "  cd android && ./gradlew assembleRelease"
