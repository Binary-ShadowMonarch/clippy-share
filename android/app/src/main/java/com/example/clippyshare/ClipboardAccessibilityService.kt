package com.example.clippyshare

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.AccessibilityServiceInfo
import android.content.ClipboardManager
import android.content.Context
import android.util.Log
import android.view.accessibility.AccessibilityEvent

class ClipboardAccessibilityService : AccessibilityService() {
    private val tag = "ClipboardAccessibility"
    private var lastText = ""
    private var clipboardManager: ClipboardManager? = null
    private var clipboardListener: ClipboardManager.OnPrimaryClipChangedListener? = null

    override fun onServiceConnected() {
        super.onServiceConnected()

        val info = AccessibilityServiceInfo().apply {
            eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
            feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
            notificationTimeout = 0
        }
        serviceInfo = info

        startClipboardMonitoring()
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // Clipboard monitoring is done through listener callbacks.
    }

    override fun onInterrupt() {
        Log.w(tag, "Accessibility service interrupted")
    }

    override fun onDestroy() {
        clipboardListener?.let { listener ->
            clipboardManager?.removePrimaryClipChangedListener(listener)
        }
        clipboardListener = null
        super.onDestroy()
    }

    private fun startClipboardMonitoring() {
        if (!RootSessionManager.hasActiveSession()) {
            Log.w(tag, "Ignoring accessibility clipboard stream because root mode is disabled")
            return
        }

        if (clipboardListener != null) {
            return
        }

        clipboardManager = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

        clipboardManager?.primaryClip?.let { clip ->
            if (clip.itemCount > 0) {
                lastText = clip.getItemAt(0).coerceToText(this).toString()
            }
        }

        clipboardListener = ClipboardManager.OnPrimaryClipChangedListener {
            try {
                val clip = clipboardManager?.primaryClip ?: return@OnPrimaryClipChangedListener
                if (clip.itemCount <= 0) {
                    return@OnPrimaryClipChangedListener
                }

                val updatedText = clip.getItemAt(0).coerceToText(this).toString()
                if (updatedText != lastText && RustBridge.shouldForwardClipboardEvent(updatedText)) {
                    lastText = updatedText
                    RustBridge.onClipboardChanged(updatedText)
                }
            } catch (e: SecurityException) {
                Log.w(tag, "Clipboard access blocked by platform policy")
            } catch (e: Exception) {
                Log.e(tag, "Clipboard event handling failed", e)
            }
        }

        clipboardManager?.addPrimaryClipChangedListener(clipboardListener)
        Log.i(tag, "Accessibility clipboard listener active")
    }
}
