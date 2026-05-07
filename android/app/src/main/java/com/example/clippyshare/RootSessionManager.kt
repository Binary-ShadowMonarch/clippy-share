package com.example.clippyshare

import java.io.IOException
import java.util.concurrent.TimeUnit

data class RootSessionResult(
    val granted: Boolean,
    val reason: String
)

object RootSessionManager {
    private const val ROOT_CHECK_TIMEOUT_SECONDS = 2L

    @Volatile
    private var activeSession = false

    fun hasActiveSession(): Boolean = activeSession

    fun requestRootSession(): RootSessionResult {
        if (!RootDetector.isLikelyRooted()) {
            activeSession = false
            return RootSessionResult(
                granted = false,
                reason = "Root not detected on this device"
            )
        }

        return runSafeRootIdentityCheck()
    }

    private fun runSafeRootIdentityCheck(): RootSessionResult {
        return try {
            val process = ProcessBuilder("su", "-c", "id")
                .redirectErrorStream(true)
                .start()

            val finished = process.waitFor(ROOT_CHECK_TIMEOUT_SECONDS, TimeUnit.SECONDS)
            if (!finished) {
                process.destroyForcibly()
                activeSession = false
                return RootSessionResult(
                    granted = false,
                    reason = "Root check timed out"
                )
            }

            val output = process.inputStream.bufferedReader().use { it.readText() }
            val granted = process.exitValue() == 0 && output.contains("uid=0")
            activeSession = granted

            if (granted) {
                RootSessionResult(granted = true, reason = "Root session granted")
            } else {
                RootSessionResult(granted = false, reason = "Root request denied")
            }
        } catch (io: IOException) {
            activeSession = false
            RootSessionResult(granted = false, reason = "Root binary unavailable")
        } catch (e: Exception) {
            activeSession = false
            RootSessionResult(granted = false, reason = "Root check failed safely")
        }
    }
}
