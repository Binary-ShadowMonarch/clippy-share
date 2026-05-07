package com.example.clippyshare

import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.os.Handler
import android.os.Looper
import android.util.Log

object RustBridge {
    private const val TAG = "RustBridge"
    private val mainHandler = Handler(Looper.getMainLooper())
    private val eventLock = Any()

    @Volatile
    private var appContext: Context? = null
    private var suppressEchoText: String? = null
    private var lastForwardedText: String? = null

    init {
        try {
            System.loadLibrary("mobile_bridge")
            Log.d(TAG, "Rust library loaded successfully")
        } catch (e: UnsatisfiedLinkError) {
            Log.e(TAG, "Failed to load Rust library: ${e.message}")
            throw RuntimeException("Native library not found. Make sure libmobile_bridge.so is in jniLibs folder")
        }
    }

    fun init(context: Context) {
        appContext = context.applicationContext
    }

    fun shouldForwardClipboardEvent(text: String): Boolean {
        synchronized(eventLock) {
            if (text.isBlank()) {
                return false
            }

            if (suppressEchoText == text) {
                suppressEchoText = null
                return false
            }

            if (lastForwardedText == text) {
                return false
            }

            lastForwardedText = text
            return true
        }
    }

    @JvmStatic
    fun applyClipboardFromRust(text: String) {
        val context = appContext
        if (context == null) {
            Log.w(TAG, "Cannot apply clipboard yet: RustBridge not initialized with context")
            return
        }

        if (!RootSessionManager.hasActiveSession()) {
            Log.w(TAG, "Ignoring remote clipboard apply because root mode is inactive")
            return
        }

        mainHandler.post {
            try {
                val manager =
                    context.getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

                synchronized(eventLock) {
                    suppressEchoText = text
                }

                manager.setPrimaryClip(ClipData.newPlainText("ClippyShare", text))
                Log.d(TAG, "Applied remote clipboard (${text.length} chars)")
            } catch (e: Exception) {
                Log.e(TAG, "Failed to apply clipboard from Rust", e)
            }
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
