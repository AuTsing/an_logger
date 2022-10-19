use android_logger::Config;
use jni::objects::GlobalRef;
use jni::objects::JValue;
use jni::JNIEnv;
use jni::JavaVM;
use log::Level;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::prelude::FromRawFd;
use std::os::unix::prelude::RawFd;
use std::thread;

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

unsafe fn redirect_to_log_write(tag: &'static str) {
    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let tag = CStr::from_bytes_with_nul(tag.as_bytes()).unwrap();
        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else if let Ok(msg) = CString::new(buffer.clone()) {
                    android_log(Level::Info, tag, &msg);
                }
            }
        }
    });
}

unsafe fn redirect_to_log_app(_tag: &'static str, env: &JNIEnv) {
    let java_vm: JavaVM;
    let io_object_global_ref: GlobalRef;
    {
        java_vm = env.get_java_vm().unwrap();
        let io_class = env.find_class("com/atstudio/denort/jni/Io").unwrap();
        let io_object = env
            .get_static_field(io_class, "INSTANCE", "Lcom/atstudio/denort/jni/Io;")
            .unwrap()
            .l()
            .unwrap();
        io_object_global_ref = env.new_global_ref(io_object).unwrap();
    }

    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let env = java_vm.attach_current_thread().unwrap();
        let io_object = io_object_global_ref.as_obj();

        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else {
                    let msg = buffer.clone();
                    let msg_trimed = msg.trim_end();
                    let msg_value: JValue = env.new_string(msg_trimed).unwrap().into();
                    env.call_method(io_object, "logInfo", "(Ljava/lang/String;)V", &[msg_value])
                        .unwrap();
                }
            }
        }
    });
}

unsafe fn redirect_to_log_write_log_app(tag: &'static str, env: &JNIEnv) {
    let java_vm: JavaVM;
    let io_object_global_ref: GlobalRef;
    {
        java_vm = env.get_java_vm().unwrap();
        let io_class = env.find_class("com/atstudio/denort/jni/Io").unwrap();
        let io_object = env
            .get_static_field(io_class, "INSTANCE", "Lcom/atstudio/denort/jni/Io;")
            .unwrap()
            .l()
            .unwrap();
        io_object_global_ref = env.new_global_ref(io_object).unwrap();
    }

    let mut logpipe: [RawFd; 2] = Default::default();
    libc::pipe(logpipe.as_mut_ptr());
    libc::dup2(logpipe[1], libc::STDOUT_FILENO);
    libc::dup2(logpipe[1], libc::STDERR_FILENO);
    thread::spawn(move || {
        let env = java_vm.attach_current_thread().unwrap();
        let io_object = io_object_global_ref.as_obj();

        let tag = CStr::from_bytes_with_nul(tag.as_bytes()).unwrap();
        let file = File::from_raw_fd(logpipe[0]);
        let mut reader = BufReader::new(file);
        let mut buffer = String::new();
        loop {
            buffer.clear();
            if let Ok(len) = reader.read_line(&mut buffer) {
                if len == 0 {
                    break;
                } else if let Ok(msg) = CString::new(buffer.clone()) {
                    android_log(Level::Info, tag, &msg);

                    let msg = buffer.clone();
                    let msg_trimed = msg.trim_end();
                    let msg_value: JValue = env.new_string(msg_trimed).unwrap().into();
                    env.call_method(io_object, "logInfo", "(Ljava/lang/String;)V", &[msg_value])
                        .unwrap();
                }
            }
        }
    });
}

pub fn init_logger_for_log_write(tag: &'static str) {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { redirect_to_log_write(tag) };
}

pub fn init_logger_for_log_app(tag: &'static str, env: &JNIEnv) {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { redirect_to_log_app(tag, env) };
}

pub fn init_logger_for_log_write_log_app(tag: &'static str, env: &JNIEnv) {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { redirect_to_log_write_log_app(tag, env) };
}
