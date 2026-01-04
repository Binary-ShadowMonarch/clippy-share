package com.example.clippyshare

import android.util.Log

object RustBridge {
    private const val TAG = "RustBridge"

    init {
        try {
            System.loadLibrary("mobile_bridge")
            Log.d(TAG, "Rust library loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load Rust library: ${e.message}")
            throw RuntimeException("Native library not found. Make sure libmobile_bridge.so is in jniLibs folder")
        }
    }

    /**
     * Starts the Rust daemon in a background thread
     */
    external fun startDaemon()

    /**
     * Called when clipboard content changes
     * @param text The new clipboard text content
     */
    external fun onClipboardChanged(text: String)

    /**
     * Optional: Get current status from Rust daemon
     * @return Status message from Rust core
     */
    external fun getStatus(): String

    /**
     * Optional: Send text to be broadcast to other devices
     * @param text Text to share with other devices
     */
    external fun shareText(text: String)
}
