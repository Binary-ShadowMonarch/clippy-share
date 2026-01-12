# clippy-share
share clipboard or files you like locally safely to any of your device in same network!

## Debugging Core Daemon Logs on Android

The Rust core daemon uses the `log` crate with `android_logger` to output logs to Android's logcat system. All Rust logs use the tag `ClippyShare` and can be viewed using `adb logcat`.

### Viewing All Core Daemon Logs

To see all logs from the core daemon and Rust bridge:

```bash
adb logcat | grep -E "ClippyShare|RustBridge"
```

Or view all logs with the ClippyShare tag:

```bash
adb logcat -s ClippyShare:D
```

### Viewing Specific Log Tags

The code uses the following log tags:
- `ClippyShare` - All Rust logs (RustBridge, CoreDaemon, AndroidAdapter)
- `RustBridge` - Kotlin logs from the RustBridge object

### Filter by Log Level

View only error logs:
```bash
adb logcat *:E | grep -E "ClippyShare|RustBridge"
```

View debug and info logs:
```bash
adb logcat *:D *:I | grep -E "ClippyShare|RustBridge"
```

Or use tag-based filtering (recommended):
```bash
# View all ClippyShare logs (Debug level and above)
adb logcat -s ClippyShare:D

# View only errors
adb logcat -s ClippyShare:E
```

### Clear Log Buffer and Monitor Fresh Logs

Clear the log buffer and start fresh:
```bash
adb logcat -c && adb logcat -s ClippyShare:D RustBridge:D
```

### Save Logs to File

Save logs to a file for later analysis:
```bash
adb logcat -s ClippyShare:D RustBridge:D > clippy-share-logs.txt
```

### Common Log Patterns to Watch For

- `startDaemon called from Kotlin` - Daemon initialization started
- `Core daemon start` - Core daemon is running
- `Received clipboard text from Kotlin` - Clipboard change detected
- `✅ RECEIVED clipboard data` - Daemon received clipboard data
- `Broadcasting encrypted data` - Data is being broadcast
