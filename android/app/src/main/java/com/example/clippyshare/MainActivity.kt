package com.example.clippyshare

import android.accessibilityservice.AccessibilityServiceInfo
import android.content.Context
import android.content.Intent
import android.os.Build
import android.os.Bundle
import android.provider.Settings
import android.util.Log
import android.view.accessibility.AccessibilityManager
import androidx.appcompat.app.AlertDialog
import androidx.appcompat.app.AppCompatActivity
import android.widget.Button
import android.widget.TextView
import android.widget.Toast
import androidx.core.app.ActivityCompat
import androidx.core.content.ContextCompat
import android.Manifest
import android.content.pm.PackageManager

class MainActivity : AppCompatActivity() {

    private val TAG = "MainActivity"
    private lateinit var statusText: TextView
    private lateinit var startButton: Button
    private lateinit var stopButton: Button
    private val NOTIFICATION_PERMISSION_REQUEST_CODE = 1001

    override fun onCreate(savedInstanceState: Bundle?) {
        super.onCreate(savedInstanceState)
        setContentView(R.layout.activity_main)

        statusText = findViewById(R.id.statusText)
        startButton = findViewById(R.id.startButton)
        stopButton = findViewById(R.id.stopButton)

        // Load Rust library
        System.loadLibrary("mobile_bridge")

        // Start Rust daemon
        RustBridge.startDaemon()

        // Request notification permission (Android 13+)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.TIRAMISU) {
            if (ContextCompat.checkSelfPermission(this, Manifest.permission.POST_NOTIFICATIONS)
                != PackageManager.PERMISSION_GRANTED
            ) {
                ActivityCompat.requestPermissions(
                    this,
                    arrayOf(Manifest.permission.POST_NOTIFICATIONS),
                    NOTIFICATION_PERMISSION_REQUEST_CODE
                )
            }
        }

        startButton.setOnClickListener {
            if (isAccessibilityServiceEnabled()) {
                startService()
            } else {
                showAccessibilityServiceDialog()
            }
        }

        stopButton.setOnClickListener {
            stopService()
        }

        checkAndUpdateStatus()
    }

    override fun onResume() {
        super.onResume()
        checkAndUpdateStatus()
    }

    private fun checkAndUpdateStatus() {
        if (isAccessibilityServiceEnabled()) {
            if (ClipboardService.isRunning) {
                updateStatus("Clipboard monitoring active")
            } else {
                updateStatus("Ready - Click Start to begin monitoring")
            }
        } else {
            updateStatus("⚠️ Accessibility permission required")
        }
    }

    private fun isAccessibilityServiceEnabled(): Boolean {
        val accessibilityManager = getSystemService(Context.ACCESSIBILITY_SERVICE) as AccessibilityManager
        val enabledServices = accessibilityManager.getEnabledAccessibilityServiceList(
            AccessibilityServiceInfo.FEEDBACK_GENERIC
        )
        return enabledServices.any { it.resolveInfo.serviceInfo.packageName == packageName }
    }

    private fun showAccessibilityServiceDialog() {
        AlertDialog.Builder(this)
            .setTitle("Enable Accessibility Service")
            .setMessage("ClippyShare needs accessibility permission to monitor clipboard changes in the background. Please enable it in the next screen.")
            .setPositiveButton("Open Settings") { _, _ ->
                val intent = Intent(Settings.ACTION_ACCESSIBILITY_SETTINGS)
                startActivity(intent)
                Toast.makeText(this, "Please enable ClippyShare in the accessibility settings", Toast.LENGTH_LONG).show()
            }
            .setNegativeButton("Cancel", null)
            .show()
    }

    private fun startService() {
        if (!isAccessibilityServiceEnabled()) {
            showAccessibilityServiceDialog()
            return
        }

        if (ClipboardService.isRunning) {
            Toast.makeText(this, "Service already running!", Toast.LENGTH_SHORT).show()
            return
        }

        val serviceIntent = Intent(this, ClipboardService::class.java)
        if (Build.VERSION.SDK_INT >= Build.VERSION_CODES.O) {
            startForegroundService(serviceIntent)
        } else {
            startService(serviceIntent)
        }
        updateStatus("Clipboard service started")
        Toast.makeText(this, "Clipboard monitoring started!", Toast.LENGTH_SHORT).show()
        Log.d(TAG, "Clipboard service started")
    }

    private fun stopService() {
        val serviceIntent = Intent(this, ClipboardService::class.java)
        stopService(serviceIntent)
        updateStatus("Clipboard service stopped")
        Toast.makeText(this, "Clipboard monitoring stopped", Toast.LENGTH_SHORT).show()
        Log.d(TAG, "Clipboard service stopped")
    }

    private fun updateStatus(message: String) {
        statusText.text = message
    }

    override fun onRequestPermissionsResult(
        requestCode: Int,
        permissions: Array<out String>,
        grantResults: IntArray
    ) {
        super.onRequestPermissionsResult(requestCode, permissions, grantResults)
        if (requestCode == NOTIFICATION_PERMISSION_REQUEST_CODE) {
            if (grantResults.isNotEmpty() && grantResults[0] == PackageManager.PERMISSION_GRANTED) {
                Log.d(TAG, "Notification permission granted")
            } else {
                Log.w(TAG, "Notification permission denied")
            }
        }
    }

    override fun onDestroy() {
        super.onDestroy()
        stopService()
    }
}
