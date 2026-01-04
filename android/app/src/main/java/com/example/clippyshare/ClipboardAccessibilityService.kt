package com.example.clippyshare

import android.accessibilityservice.AccessibilityService
import android.accessibilityservice.AccessibilityServiceInfo
import android.content.ClipData
import android.content.ClipboardManager
import android.content.Context
import android.util.Log
import android.view.accessibility.AccessibilityEvent

class ClipboardAccessibilityService : AccessibilityService() {
    private val TAG = "ClipboardAccessibility"
    private var lastClipText = ""

    // #region agent log
    private fun logDebug(sessionId: String, runId: String, hypothesisId: String, location: String, message: String, data: Map<String, Any?> = emptyMap()) {
        val dataStr = data.entries.joinToString(", ") { "${it.key}=${it.value}" }
        Log.d(TAG, "[$hypothesisId] $location: $message | $dataStr")
    }
    // #endregion agent log

    override fun onServiceConnected() {
        super.onServiceConnected()
        Log.d(TAG, "Accessibility service connected")
        // #region agent log
        logDebug("debug-session", "run1", "A", "ClipboardAccessibilityService.kt:onServiceConnected", "Service connected", mapOf("service" to "ClipboardAccessibilityService"))
        // #endregion agent log
        
        val info = AccessibilityServiceInfo().apply {
            eventTypes = AccessibilityEvent.TYPE_WINDOW_STATE_CHANGED
            feedbackType = AccessibilityServiceInfo.FEEDBACK_GENERIC
            flags = AccessibilityServiceInfo.FLAG_INCLUDE_NOT_IMPORTANT_VIEWS
            notificationTimeout = 0
        }
        setServiceInfo(info)
        
        // Start monitoring clipboard
        startClipboardMonitoring()
    }

    override fun onAccessibilityEvent(event: AccessibilityEvent?) {
        // We use this service primarily for clipboard access, not for accessibility events
        // But we can monitor window changes to detect when clipboard might have changed
    }

    override fun onInterrupt() {
        Log.d(TAG, "Accessibility service interrupted")
    }

    private fun startClipboardMonitoring() {
        val clipboardManager = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager
        
        // #region agent log
        logDebug("debug-session", "run1", "B", "ClipboardAccessibilityService.kt:startClipboardMonitoring", "Starting clipboard monitoring", mapOf("service" to "ClipboardAccessibilityService"))
        // #endregion agent log
        
        // Get initial clipboard content
        try {
            val currentClip = clipboardManager.primaryClip
            // #region agent log
            logDebug("debug-session", "run1", "C", "ClipboardAccessibilityService.kt:startClipboardMonitoring", "Initial clipboard read attempt", mapOf("hasClip" to (currentClip != null), "itemCount" to (currentClip?.itemCount ?: 0)))
            // #endregion agent log
            
            if (currentClip != null && currentClip.itemCount > 0) {
                val text = currentClip.getItemAt(0).coerceToText(this).toString()
                if (text.isNotEmpty()) {
                    lastClipText = text
                    Log.d(TAG, "Initial clipboard content: ${text.take(50)}...")
                    // #region agent log
                    logDebug("debug-session", "run1", "D", "ClipboardAccessibilityService.kt:startClipboardMonitoring", "Initial clipboard content captured", mapOf("textLength" to text.length, "textPreview" to text.take(50)))
                    // #endregion agent log
                }
            }
        } catch (e: SecurityException) {
            Log.e(TAG, "Security exception reading initial clipboard: ${e.message}", e)
            // #region agent log
            logDebug("debug-session", "run1", "E", "ClipboardAccessibilityService.kt:startClipboardMonitoring", "Security exception on initial read", mapOf("error" to e.message, "errorType" to "SecurityException"))
            // #endregion agent log
        } catch (e: Exception) {
            Log.e(TAG, "Error reading initial clipboard: ${e.message}", e)
            // #region agent log
            logDebug("debug-session", "run1", "F", "ClipboardAccessibilityService.kt:startClipboardMonitoring", "Exception on initial read", mapOf("error" to e.message, "errorType" to e.javaClass.simpleName))
            // #endregion agent log
        }

        // Set up clipboard listener
        clipboardManager.addPrimaryClipChangedListener {
            try {
                // #region agent log
                logDebug("debug-session", "run1", "G", "ClipboardAccessibilityService.kt:onClipboardChanged", "Clipboard change detected", mapOf("listenerActive" to true))
                // #endregion agent log
                
                val clip = clipboardManager.primaryClip
                // #region agent log
                logDebug("debug-session", "run1", "H", "ClipboardAccessibilityService.kt:onClipboardChanged", "Clipboard read attempt", mapOf("hasClip" to (clip != null), "itemCount" to (clip?.itemCount ?: 0)))
                // #endregion agent log
                
                if (clip != null && clip.itemCount > 0) {
                    val newText = clip.getItemAt(0).coerceToText(this).toString()
                    // #region agent log
                    logDebug("debug-session", "run1", "I", "ClipboardAccessibilityService.kt:onClipboardChanged", "Clipboard text extracted", mapOf("textLength" to newText.length, "isDifferent" to (newText != lastClipText), "textPreview" to newText.take(50)))
                    // #endregion agent log
                    
                    if (newText.isNotEmpty() && newText != lastClipText) {
                        lastClipText = newText
                        Log.d(TAG, "Clipboard changed: ${newText.take(50)}...")
                        // #region agent log
                        logDebug("debug-session", "run1", "J", "ClipboardAccessibilityService.kt:onClipboardChanged", "Sending to Rust daemon", mapOf("textLength" to newText.length))
                        // #endregion agent log
                        
                        // Send to Rust daemon
                        RustBridge.onClipboardChanged(newText)
                        
                        // #region agent log
                        logDebug("debug-session", "run1", "K", "ClipboardAccessibilityService.kt:onClipboardChanged", "Sent to Rust daemon", mapOf("success" to true))
                        // #endregion agent log
                    }
                }
            } catch (e: SecurityException) {
                Log.e(TAG, "Security exception reading clipboard: ${e.message}", e)
                // #region agent log
                logDebug("debug-session", "run1", "L", "ClipboardAccessibilityService.kt:onClipboardChanged", "Security exception", mapOf("error" to e.message, "errorType" to "SecurityException"))
                // #endregion agent log
            } catch (e: Exception) {
                Log.e(TAG, "Error reading clipboard: ${e.message}", e)
                // #region agent log
                logDebug("debug-session", "run1", "M", "ClipboardAccessibilityService.kt:onClipboardChanged", "Exception reading clipboard", mapOf("error" to e.message, "errorType" to e.javaClass.simpleName))
                // #endregion agent log
            }
        }
        
        Log.d(TAG, "Clipboard monitoring started")
        // #region agent log
        logDebug("debug-session", "run1", "N", "ClipboardAccessibilityService.kt:startClipboardMonitoring", "Clipboard monitoring initialized", mapOf("listenerAdded" to true))
        // #endregion agent log
    }
}

