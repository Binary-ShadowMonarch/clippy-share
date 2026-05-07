package com.example.clippyshare

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.ClipboardManager
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import android.util.Log
import androidx.core.app.NotificationCompat

class ClipboardService : Service() {
    private val tag = "ClipboardService"

    private val notificationId = 1
    private val channelId = "clipboard_service_channel"
    private val channelName = "Clipboard Monitor"
    private var clipboardManager: ClipboardManager? = null
    private var clipboardListener: ClipboardManager.OnPrimaryClipChangedListener? = null
    private var lastText = ""

    companion object {
        var isRunning = false
    }

    override fun onCreate() {
        super.onCreate()
        createNotificationChannel()
        startForeground(notificationId, createNotification())
        isRunning = true
    }

    override fun onStartCommand(intent: Intent?, flags: Int, startId: Int): Int {
        startClipboardMonitoring()
        return START_STICKY
    }

    override fun onBind(intent: Intent?): IBinder? = null

    override fun onDestroy() {
        clipboardListener?.let { listener ->
            clipboardManager?.removePrimaryClipChangedListener(listener)
        }
        clipboardListener = null
        stopForeground(STOP_FOREGROUND_REMOVE)
        isRunning = false
        super.onDestroy()
    }

    private fun createNotificationChannel() {
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            val channel = NotificationChannel(
                channelId,
                channelName,
                NotificationManager.IMPORTANCE_LOW
            ).apply {
                description = "Monitors clipboard changes and syncs with other devices"
                lockscreenVisibility = Notification.VISIBILITY_PUBLIC
            }
            val manager = getSystemService(Context.NOTIFICATION_SERVICE) as NotificationManager
            manager.createNotificationChannel(channel)
        }
    }

    private fun createNotification(): Notification {
        return NotificationCompat.Builder(this, channelId)
            .setContentTitle("ClippyShare")
            .setContentText("Background clipboard sync is active")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    private fun startClipboardMonitoring() {
        if (!RootSessionManager.hasActiveSession()) {
            Log.w(tag, "Skipping background monitoring: root session is not active")
            stopSelf()
            return
        }

        if (clipboardListener != null) {
            return
        }

        clipboardManager = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

        // Initial clipboard content
        val currentClip = clipboardManager?.primaryClip
        if (currentClip != null && currentClip.itemCount > 0) {
            lastText = currentClip.getItemAt(0).coerceToText(this).toString()
        }

        clipboardListener = ClipboardManager.OnPrimaryClipChangedListener {
            try {
                val newClip = clipboardManager?.primaryClip ?: return@OnPrimaryClipChangedListener
                if (newClip.itemCount <= 0) {
                    return@OnPrimaryClipChangedListener
                }

                val newText = newClip.getItemAt(0).coerceToText(this).toString()
                if (newText != lastText && RustBridge.shouldForwardClipboardEvent(newText)) {
                    lastText = newText
                    RustBridge.onClipboardChanged(newText)
                }
            } catch (e: SecurityException) {
                Log.w(tag, "Clipboard access blocked by platform policy")
            } catch (e: Exception) {
                Log.e(tag, "Failed to process clipboard update", e)
            }
        }

        clipboardManager?.addPrimaryClipChangedListener(clipboardListener)
        Log.i(tag, "Event-driven clipboard listener started")
    }
}
