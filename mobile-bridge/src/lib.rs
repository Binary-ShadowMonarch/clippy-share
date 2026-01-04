#![allow(non_snake_case)]
#![cfg(target_os = "android")]

use core_daemon::CoreDaemon;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use platform_adapters::{ClipData, android::CLIP_SENDER};
use std::thread;

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clippyshare_RustBridge_startDaemon(
    _env: JNIEnv<'_>,
    _class: JClass<'_>,
) {
    eprintln!("[RustBridge] startDaemon called from Kotlin");
    thread::spawn(|| {
        eprintln!("[RustBridge] Starting daemon in background thread");
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            eprintln!("[RustBridge] Creating CoreDaemon");
            let daemon = CoreDaemon::new();
            eprintln!("[RustBridge] Starting CoreDaemon.run()");
            daemon.run().await;
        });
    });
    eprintln!("[RustBridge] Daemon thread spawned");
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clippyshare_RustBridge_onClipboardChanged(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    j_text: JString<'_>,
) {
    let text: String = match env.get_string(&j_text) {
        Ok(java_str) => java_str.into(),
        Err(e) => {
            eprintln!("Failed to convert JString to Rust String: {:?}", e);
            return;
        }
    };

    let clip = ClipData::Text(text.clone()); // clone for UI if needed, but mainly for daemon

    // #region agent log
    eprintln!("[RustBridge] Received clipboard text from Kotlin, length: {}", text.len());
    // #endregion agent log

    let guard = CLIP_SENDER.lock().expect("CLIP_SENDER mutex poisoned");
    if let Some(tx) = guard.as_ref() {
        // #region agent log
        eprintln!("[RustBridge] Sending clipboard data to daemon channel");
        // #endregion agent log
        if let Err(e) = tx.send(clip) {
            eprintln!("[RustBridge] Failed to send clipboard data to daemon: {}", e);
        } else {
            // #region agent log
            eprintln!("[RustBridge] Successfully sent clipboard data to daemon channel");
            // #endregion agent log
        }
    } else {
        eprintln!("[RustBridge] Sender not available – daemon probably not started yet");
    }
}
