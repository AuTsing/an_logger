use android_logger::Config;
use log::Level;
use once_cell::sync::Lazy;
use std::ffi::CStr;
use std::ffi::CString;
use std::fs::File;
use std::io::BufRead;
use std::io::BufReader;
use std::os::unix::prelude::FromRawFd;
use std::os::unix::prelude::RawFd;
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

fn try_log_write(buffer: &String) {
    if let Ok(msg) = CString::new(buffer.clone()) {
        android_log(Level::Info, &TAG, &msg);
    }
}

unsafe fn start_redirect() {
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
                    try_log_write(&buffer);
                }
            }
        }
    });
}

unsafe fn start_redirect_only<F>(f: F)
where
    F: Fn(&String) + Send + 'static,
{
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
                    f(&buffer);
                }
            }
        }
    });
}

unsafe fn start_redirect_also<F>(f: F)
where
    F: Fn(&String) + Send + 'static,
{
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
                    try_log_write(&buffer);
                    f(&buffer);
                }
            }
        }
    });
}

pub fn init_logger() {
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { start_redirect() };
}

pub fn init_logger_only<F>(f: F)
where
    F: Fn(&String) + Send + 'static,
{
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { start_redirect_only(f) };
}

pub fn init_logger_also<F>(f: F)
where
    F: Fn(&String) + Send + 'static,
{
    android_logger::init_once(Config::default().with_min_level(Level::Trace));
    unsafe { start_redirect_also(f) };
}
