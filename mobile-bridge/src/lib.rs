#![allow(non_snake_case)]
#![cfg(target_os = "android")]

use core_daemon::CoreDaemon;
use jni::JNIEnv;
use jni::objects::{JClass, JString, JValue};
use jni::sys::jstring;
use jni::JavaVM;
use once_cell::sync::OnceCell;
use platform_adapters::{android, android::CLIP_SENDER, ClipData};
use std::sync::atomic::{AtomicBool, Ordering};
use std::thread;
use std::sync::Once;
use log::LevelFilter;

static LOGGER_INIT: Once = Once::new();
static APPLY_FORWARDER_INIT: Once = Once::new();
static DAEMON_STARTED: AtomicBool = AtomicBool::new(false);

static JAVA_VM: OnceCell<JavaVM> = OnceCell::new();

fn init_logger(env: &JNIEnv) {
    LOGGER_INIT.call_once(|| {
        // Get JavaVM - android_logger needs this internally
        let java_vm = env.get_java_vm().expect("Failed to get JavaVM");
        let _ = JAVA_VM.set(java_vm);
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

    if DAEMON_STARTED.swap(true, Ordering::SeqCst) {
        log::warn!("startDaemon called more than once; ignoring duplicate start");
        return;
    }

    let (apply_tx, apply_rx) = crossbeam_channel::unbounded::<String>();
    android::set_apply_sender(apply_tx);

    APPLY_FORWARDER_INIT.call_once(|| {
        thread::spawn(move || {
            while let Ok(text) = apply_rx.recv() {
                if let Err(err) = apply_text_on_android_main(text) {
                    log::error!("Failed to apply remote clipboard on Android: {}", err);
                }
            }
        });
    });

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

fn apply_text_on_android_main(text: String) -> Result<(), String> {
    let java_vm = JAVA_VM
        .get()
        .ok_or_else(|| "JavaVM is not initialized".to_string())?;

    let mut env = java_vm
        .attach_current_thread()
        .map_err(|err| format!("Failed to attach JNI thread: {err:?}"))?;

    let j_text = env
        .new_string(text)
        .map_err(|err| format!("Failed to create Java string: {err:?}"))?;

    env.call_static_method(
        "com/example/clippyshare/RustBridge",
        "applyClipboardFromRust",
        "(Ljava/lang/String;)V",
        &[JValue::Object(&j_text)],
    )
    .map_err(|err| format!("Failed to call RustBridge.applyClipboardFromRust: {err:?}"))?;

    Ok(())
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

#[unsafe(no_mangle)]
pub extern "system" fn Java_com_example_clippyshare_RustBridge_getStatus(
    mut env: JNIEnv<'_>,
    _class: JClass<'_>,
) -> jstring {
    init_logger(&env);
    let status = if DAEMON_STARTED.load(Ordering::SeqCst) {
        "daemon_running"
    } else {
        "daemon_not_started"
    };

    match env.new_string(status) {
        Ok(s) => s.into_raw(),
        Err(_) => std::ptr::null_mut(),
    }
}
