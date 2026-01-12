#![allow(non_snake_case)]
#![cfg(target_os = "android")]

use core_daemon::CoreDaemon;
use jni::JNIEnv;
use jni::objects::{JClass, JString};
use platform_adapters::{ClipData, android::CLIP_SENDER};
use std::thread;
use std::sync::Once;
use log::LevelFilter;

static LOGGER_INIT: Once = Once::new();

fn init_logger(env: &JNIEnv) {
    LOGGER_INIT.call_once(|| {
        // Get JavaVM - android_logger needs this internally
        let _java_vm = env.get_java_vm().expect("Failed to get JavaVM");
        android_logger::init_once(
            android_logger::Config::default()
                .with_max_level(LevelFilter::Debug)
                .with_tag("ClippyShare")
        );
    });
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clippyshare_RustBridge_startDaemon(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) {
    init_logger(&env);
    log::info!("startDaemon called from Kotlin");
    thread::spawn(|| {
        log::info!("Starting daemon in background thread");
        let rt = tokio::runtime::Runtime::new().expect("Failed to create Tokio runtime");
        rt.block_on(async {
            log::info!("Creating CoreDaemon");
            let daemon = CoreDaemon::new();
            log::info!("Starting CoreDaemon.run()");
            daemon.run().await;
        });
    });
    log::info!("Daemon thread spawned");
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clippyshare_RustBridge_onClipboardChanged(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    j_text: JString<'_>,
) {
    init_logger(&env);
    let text: String = match env.get_string(&j_text) {
        Ok(java_str) => java_str.into(),
        Err(e) => {
            log::error!("Failed to convert JString to Rust String: {:?}", e);
            return;
        }
    };

    let clip = ClipData::Text(text.clone()); // clone for UI if needed, but mainly for daemon

    log::info!("Received clipboard text from Kotlin, length: {}", text.len());

    let guard = CLIP_SENDER.lock().expect("CLIP_SENDER mutex poisoned");
    if let Some(tx) = guard.as_ref() {
        log::debug!("Sending clipboard data to daemon channel");
        if let Err(e) = tx.send(clip) {
            log::error!("Failed to send clipboard data to daemon: {}", e);
        } else {
            log::debug!("Successfully sent clipboard data to daemon channel");
        }
    } else {
        log::warn!("Sender not available – daemon probably not started yet");
    }
}

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clippyshare_RustBridge_shareText(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
    j_text: JString<'_>,
) {
    init_logger(&env);
    let text: String = match env.get_string(&j_text) {
        Ok(java_str) => java_str.into(),
        Err(e) => {
            log::error!("Failed to convert JString to Rust String: {:?}", e);
            return;
        }
    };

    let clip = ClipData::Text(text.clone());

    log::info!("shareText called from Kotlin, text length: {}", text.len());

    let guard = CLIP_SENDER.lock().expect("CLIP_SENDER mutex poisoned");
    if let Some(tx) = guard.as_ref() {
        log::debug!("Sending text to daemon channel for broadcasting");
        if let Err(e) = tx.send(clip) {
            log::error!("Failed to send text to daemon: {}", e);
        } else {
            log::debug!("Successfully sent text to daemon channel");
        }
    } else {
        log::warn!("Sender not available – daemon probably not started yet");
    }
}
