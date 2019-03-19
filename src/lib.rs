use std::sync::Mutex;

use detour::GenericDetour;
use lazy_static::lazy_static;
use log::info;
use widestring::{U16CStr, U16CString};
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::*;

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    const DLL_PROCESS_ATTACH: DWORD = 1;
    const DLL_PROCESS_DETACH: DWORD = 0;

    match call_reason {
        DLL_PROCESS_ATTACH => init(),
        DLL_PROCESS_DETACH => (),
        _ => (),
    }

    return TRUE;
}

type ConsoleWriteFn = extern "cdecl" fn(TextColor, *const wchar_t) -> BOOL;

// Detour Setup
lazy_static! {
    static ref DETOUR_CONSOLE_WRITE: Mutex<GenericDetour<ConsoleWriteFn>> = {
        unsafe {
            Mutex::new(
                GenericDetour::<ConsoleWriteFn>::new(
                    std::mem::transmute(0x00450b90),
                    detour_console_write,
                )
                .unwrap(),
            )
        }
    };
}

lazy_static! {
    static ref LAST_MESSAGE_HAX: Mutex<bool> = Mutex::new(false);
}

#[derive(Debug)]
#[repr(u32)]
pub enum TextColor {
    Black = 1,
    Grey = 2,
    White = 3,
    White2 = 4,
    DarkRed = 5,
    Red = 6,
    LightRed = 7,
    DarkGreen = 8,
    Green = 9,
    LightGreen = 10,
    DarkBlue = 11,
    Blue = 12,
    LightBlue = 13,
    DarkYellow = 14,
    Yellow = 15,
    LightYellow = 16,
}

extern "cdecl" fn detour_console_write(color: TextColor, message: *const wchar_t) -> BOOL {
    let realfn: ConsoleWriteFn =
        unsafe { std::mem::transmute(DETOUR_CONSOLE_WRITE.lock().unwrap().trampoline()) };

    // Convert message to a utf8 string
    let s = unsafe { U16CStr::from_ptr_str(message).to_string_lossy() };
    info!(
        "color = {:?} | message = {:?} | *message = {}",
        color, message, s
    );

    if *LAST_MESSAGE_HAX.lock().unwrap() {
        *LAST_MESSAGE_HAX.lock().unwrap() = false;

        return TRUE;
    }

    if s.starts_with("> /hax") {
        *LAST_MESSAGE_HAX.lock().unwrap() = true;
        if &s[6..] == " help" {
            let hax_usage = r#"first line
second line
third line"#;

            hax_usage.split("\n").for_each(|line| {
                let u16cstring = U16CString::from_str(line).unwrap();
                realfn(TextColor::LightBlue, u16cstring.as_ptr());
            });
        }

        return TRUE;
    }

    return realfn(color, message);
}

fn init() {
    // Give us a console window to write to
    unsafe { winapi::um::consoleapi::AllocConsole() };
    simple_logger::init().unwrap();

    info!("Initializing...");

    // Enable hooks
    unsafe {
        DETOUR_CONSOLE_WRITE.lock().unwrap().enable().unwrap();
    }
}
