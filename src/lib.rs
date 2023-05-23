use android_logger::Config;
use jni::errors::Error;
use jni::objects::GlobalRef;
use jni::objects::JMethodID;
use jni::objects::JValue;
use jni::signature::Primitive;
use jni::signature::ReturnType;
use jni::Executor;
use jni::JNIEnv;
use log::Level;
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::prelude::FromRawFd;
use std::os::unix::prelude::RawFd;
use std::sync::Arc;
use std::thread;

static TAG: Lazy<&CStr> = Lazy::new(|| CStr::from_bytes_with_nul(b"AnLogger\0").unwrap());

#[cfg(target_os = "android")]
fn android_log(level: Level, tag: &CStr, msg: &CStr) {
    let prio = match level {
        Level::Error => ndk_sys::android_LogPriority::ANDROID_LOG_ERROR,
        Level::Warn => ndk_sys::android_LogPriority::ANDROID_LOG_WARN,
        Level::Info => ndk_sys::android_LogPriority::ANDROID_LOG_INFO,
        Level::Debug => ndk_sys::android_LogPriority::ANDROID_LOG_DEBUG,
        Level::Trace => ndk_sys::android_LogPriority::ANDROID_LOG_VERBOSE,
    };
    unsafe {
        ndk_sys::__android_log_write(prio.0 as std::os::raw::c_int, tag.as_ptr(), msg.as_ptr());
    }
}

#[cfg(not(target_os = "android"))]
fn android_log(_level: Level, _tag: &CStr, _msg: &CStr) {}

fn try_log_write(buffer: String) {
    if let Ok(msg) = CString::new(buffer) {
        android_log(Level::Info, &TAG, &msg);
    }
}

fn try_log_app(
    exec: &Executor,
    io_globalref: &GlobalRef,
    io_log_info_jmethodid: &JMethodID,
    buffer: String,
) {
    exec.with_attached_capacity(4, |env| {
        let msg = buffer.trim_end();
        let msg_jstring = env.new_string(msg)?;
        unsafe {
            env.call_method_unchecked(
                io_globalref,
                io_log_info_jmethodid,
                ReturnType::Primitive(Primitive::Void),
                &[JValue::from(&msg_jstring).as_jni()],
            )?;
        }

        Ok::<(), Error>(())
    })
    .unwrap();
}

unsafe fn redirect_to_log_write() {
    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else {
                    try_log_write(buffer.clone());
                }
            }
        }
    });
}

unsafe fn redirect_to_log_app(env: &mut JNIEnv) {
    let exec: Executor;
    let io_globalref: GlobalRef;
    let io_log_info_jmethodid: JMethodID;
    {
        let jvm = env.get_java_vm().unwrap();
        exec = Executor::new(Arc::new(jvm));
        let io_jclass = env.find_class("com/atstudio/denort/jni/Io").unwrap();
        let io_jobject = env
            .get_static_field(&io_jclass, "INSTANCE", "Lcom/atstudio/denort/jni/Io;")
            .unwrap()
            .l()
            .unwrap();
        io_globalref = env.new_global_ref(io_jobject).unwrap();
        io_log_info_jmethodid = env
            .get_method_id(&io_jclass, "logInfo", "(Ljava/lang/String;)V")
            .unwrap();
    }

    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else {
                    try_log_app(&exec, &io_globalref, &io_log_info_jmethodid, buffer.clone());
                }
            }
        }
    });
}

unsafe fn redirect_to_log_write_log_app(env: &mut JNIEnv) {
    let exec: Executor;
    let io_globalref: GlobalRef;
    let io_log_info_jmethodid: JMethodID;
    {
        let jvm = env.get_java_vm().unwrap();
        exec = Executor::new(Arc::new(jvm));
        let io_jclass = env.find_class("com/atstudio/denort/jni/Io").unwrap();
        let io_jobject = env
            .get_static_field(&io_jclass, "INSTANCE", "Lcom/atstudio/denort/jni/Io;")
            .unwrap()
            .l()
            .unwrap();
        io_globalref = env.new_global_ref(io_jobject).unwrap();
        io_log_info_jmethodid = env
            .get_method_id(&io_jclass, "logInfo", "(Ljava/lang/String;)V")
            .unwrap();
    }

    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else {
                    try_log_write(buffer.clone());
                    try_log_app(&exec, &io_globalref, &io_log_info_jmethodid, buffer.clone());
                }
            }
        }
    });
}

pub fn init_logger_for_log_write() {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { redirect_to_log_write() };
}

pub fn init_logger_for_log_app(env: &mut JNIEnv) {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { redirect_to_log_app(env) };
}

pub fn init_logger_for_log_write_log_app(env: &mut JNIEnv) {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { redirect_to_log_write_log_app(env) };
}
