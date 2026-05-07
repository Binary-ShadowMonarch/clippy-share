package com.example.clippyshare

import android.os.Build
import java.io.File

object RootDetector {
    private val suPaths = listOf(
        "/system/bin/su",
        "/system/xbin/su",
        "/sbin/su",
        "/su/bin/su",
        "/data/local/xbin/su",
        "/data/local/bin/su"
    )

    private val rootMarkers = listOf(
        "/data/adb/magisk",
        "/system/app/Superuser.apk"
    )

    fun hasSuBinary(): Boolean {
        return suPaths.any { path ->
            val file = File(path)
            file.exists() && file.canExecute()
        }
    }

    fun hasRootMarkers(): Boolean {
        return rootMarkers.any { marker -> File(marker).exists() }
    }

    fun hasTestKeysBuild(): Boolean {
        return Build.TAGS?.contains("test-keys") == true
    }

    fun isLikelyRooted(): Boolean {
        return hasSuBinary() || hasRootMarkers() || hasTestKeysBuild()
    }
}
