package com.example.clippyshare

import android.Manifest
import android.content.ClipboardManager
import android.content.Intent
import android.content.pm.PackageManager
import android.os.Build
import android.os.Bundle
import android.util.Log
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import kotlinx.coroutines.CoroutineScope
import kotlinx.coroutines.Dispatchers
import kotlinx.coroutines.launch
import kotlinx.coroutines.withContext

class MainActivity : AppCompatActivity() {
    private val tag = "MainActivity"
    private val notificationPermissionRequestCode = 1001

    private lateinit var modeText: TextView
    private lateinit var statusText: TextView
    private lateinit var startButton: Button
    private lateinit var stopButton: Button
    private lateinit var sendClipboardButton: Button

    private var rootGranted = false
    private var rootReason = ""
    private var noRootDialogShown = false

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        modeText = findViewById(R.id.modeText)
        statusText = findViewById(R.id.statusText)
        startButton = findViewById(R.id.startButton)
        stopButton = findViewById(R.id.stopButton)
        sendClipboardButton = findViewById(R.id.sendClipboardButton)

        RustBridge.init(applicationContext)
        RustBridge.startDaemon()

        requestNotificationPermissionIfNeeded()
        checkRootModeOnce()

        startButton.setOnClickListener {
            startBackgroundMonitoring()
        }

        stopButton.setOnClickListener {
            stopBackgroundMonitoring()
        }

        sendClipboardButton.setOnClickListener {
            sendClipboardNow()
        }

        updateStatus()
    }

    override fun onResume() {
        super.onResume()
        updateStatus()
    }

    private fun requestNotificationPermissionIfNeeded() {
        if (Build.VERSION.SDK_INT < Build.VERSION_CODES.TIRAMISU) {
            return
        }

        if (
            ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
            != PackageManager.PERMISSION_GRANTED
        ) {
            ActivityCompat.requestPermissions(
                this,
                arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                notificationPermissionRequestCode
            )
        }
    }

    private fun checkRootModeOnce() {
        CoroutineScope(Dispatchers.IO).launch {
            val result = RootSessionManager.requestRootSession()
            withContext(Dispatchers.Main) {
                rootGranted = result.granted
                rootReason = result.reason

                if (!rootGranted) {
                    showNoRootDialog(result.reason)
                }

                updateStatus()
            }
        }
    }

    private fun startBackgroundMonitoring() {
        if (!rootGranted) {
            showNoRootDialog(rootReason)
            return
        }

        if (ClipboardService.isRunning) {
            Toast.makeText(this, R.string.service_already_running, Toast.LENGTH_SHORT).show()
            updateStatus()
            return
        }

        val intent = Intent(this, ClipboardService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(intent)
        } else {
            startService(intent)
        }

        Toast.makeText(this, R.string.service_started, Toast.LENGTH_SHORT).show()
        updateStatus()
    }

    private fun stopBackgroundMonitoring() {
        val intent = Intent(this, ClipboardService::class.java)
        stopService(intent)
        Toast.makeText(this, R.string.service_stopped, Toast.LENGTH_SHORT).show()
        updateStatus()
    }

    private fun sendClipboardNow() {
        RustBridge.init(applicationContext)

        val clipboardManager = getSystemService(CLIPBOARD_SERVICE) as ClipboardManager
        val clipData = clipboardManager.primaryClip

        if (clipData == null || clipData.itemCount <= 0) {
            Toast.makeText(this, R.string.no_clipboard_content, Toast.LENGTH_SHORT).show()
            return
        }

        val text = clipData.getItemAt(0).coerceToText(this).toString()
        if (text.isBlank()) {
            Toast.makeText(this, R.string.empty_clipboard, Toast.LENGTH_SHORT).show()
            return
        }

        RustBridge.shareText(text)
        Toast.makeText(this, R.string.clipboard_sent, Toast.LENGTH_SHORT).show()
    }

    private fun showNoRootDialog(reason: String) {
        if (noRootDialogShown) {
            return
        }
        noRootDialogShown = true

        AlertDialog.Builder(this)
            .setTitle(R.string.no_root_title)
            .setMessage(
                getString(
                    R.string.no_root_message,
                    reason.ifBlank { getString(R.string.no_root_reason_unknown) }
                )
            )
            .setPositiveButton(R.string.ok, null)
            .show()
    }

    private fun updateStatus() {
        if (rootGranted) {
            modeText.text = getString(R.string.mode_root_active)
            statusText.text = if (ClipboardService.isRunning) {
                getString(R.string.status_root_running)
            } else {
                getString(R.string.status_root_ready)
            }
        } else {
            modeText.text = getString(R.string.mode_manual_only)
            statusText.text = getString(R.string.status_manual_only)
        }
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)

        if (requestCode == notificationPermissionRequestCode) {
            if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                Log.d(tag, "Notification permission granted")
            } else {
                Log.w(tag, "Notification permission denied")
            }
        }
    }
}
