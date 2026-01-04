package com.example.clippyshare

import android.app.Notification
import android.app.NotificationChannel
import android.app.NotificationManager
import android.app.Service
import android.content.ClipboardManager
import android.content.ClipData
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.IBinder
import androidx.core.app.NotificationCompat
import kotlinx.coroutines.*
import kotlinx.coroutines.sync.Mutex
import kotlinx.coroutines.sync.withLock

class ClipboardService : Service() {

    private val clipboardJob: Job
    private val notificationId = 1
    private val channelId = "clipboard_service_channel"
    private val channelName = "Clipboard Monitor"
    private val mutex = Mutex()
    private var lastText = ""

    companion object {
        var isRunning = false
    }

    init {
        clipboardJob = SupervisorJob()
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
        clipboardJob.cancel()
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
            .setContentText("Monitoring clipboard changes...")
            .setSmallIcon(android.R.drawable.ic_dialog_info)
            .setOngoing(true)
            .setPriority(NotificationCompat.PRIORITY_LOW)
            .build()
    }

    private fun startClipboardMonitoring() {
        val clipboardManager = getSystemService(Context.CLIPBOARD_SERVICE) as ClipboardManager

        // Initial clipboard content
        val currentClip = clipboardManager.primaryClip
        if (currentClip != null && currentClip.itemCount > 0) {
            lastText = currentClip.getItemAt(0).coerceToText(this).toString()
        }

        CoroutineScope(Dispatchers.Main + clipboardJob).launch {
            while (isActive) {
                try {
                    delay(500) // Check every 500ms to avoid battery drain

                    mutex.withLock {
                        val newClip = clipboardManager.primaryClip
                        if (newClip != null && newClip.itemCount > 0) {
                            val newText = newClip.getItemAt(0).coerceToText(this@ClipboardService).toString()

                            if (newText.isNotEmpty() && newText != lastText) {
                                lastText = newText
                                // Send to Rust daemon
                                RustBridge.onClipboardChanged(newText)
                            }
                        }
                    }
                } catch (e: Exception) {
                    // Continue monitoring even if there's an error
                }
            }
        }
    }
}
