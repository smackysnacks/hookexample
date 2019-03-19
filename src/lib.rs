use std::sync::Mutex;

use detour::GenericDetour;
use lazy_static::lazy_static;
use log::info;
use widestring::{U16CStr, U16CString};
use winapi::ctypes::wchar_t;
use winapi::shared::minwindef::*;
use winapi::um::winnt::{DLL_PROCESS_ATTACH, DLL_PROCESS_DETACH};

#[no_mangle]
#[allow(non_snake_case, unused_variables)]
pub extern "system" fn DllMain(
    dll_module: HINSTANCE,
    call_reason: DWORD,
    reserved: LPVOID,
) -> BOOL {
    match call_reason {
        DLL_PROCESS_ATTACH => on_attach(),
        DLL_PROCESS_DETACH => on_detach(),
        _ => (),
    }

    return TRUE;
}

macro_rules! setup_hook {
    ( $name:ident, $fntype:ty, $originalfn:expr, $detourfn:expr ) => {
        lazy_static! {
            static ref $name: Mutex<GenericDetour<$fntype>> = unsafe {
                Mutex::new(
                    GenericDetour::<$fntype>::new(std::mem::transmute($originalfn), $detourfn)
                        .unwrap(),
                )
            };
        }
    };
}

macro_rules! enable_hook {
    ( $name:ident ) => {
        unsafe { $name.lock().unwrap().enable().unwrap() };
    };
}

macro_rules! disable_hook {
    ( $name:ident ) => {
        unsafe { $name.lock().unwrap().disable().unwrap() };
    };
}

// Function signatures
type ConsoleWriteFn = extern "cdecl" fn(TextColor, *const wchar_t) -> BOOL;

// Hook setup
setup_hook!(DETOUR_CONSOLE_WRITE, ConsoleWriteFn, 0x00450b90, detour_console_write);

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
            let hax_usage = ["first line", "second line", "third line"].join("\n");

            hax_usage.split("\n").for_each(|line| {
                let u16cstring = U16CString::from_str(line).unwrap();
                realfn(TextColor::LightBlue, u16cstring.as_ptr());
            });
        }

        return TRUE;
    }

    return realfn(color, message);
}

fn on_attach() {
    // Give us a console window to write to
    unsafe { winapi::um::consoleapi::AllocConsole() };

    // Create a simple logger so we can use debug, info, error and friends
    simple_logger::init().unwrap();

    info!("Setting up hooks...");

    // Enable hooks
    enable_hook!(DETOUR_CONSOLE_WRITE);
}

fn on_detach() {
    info!("Tearing down hooks...");

    // Disable hooks
    disable_hook!(DETOUR_CONSOLE_WRITE);

    // Detach the console
    unsafe { winapi::um::wincon::FreeConsole() };
}
