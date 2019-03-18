use std::sync::Mutex;

use detour::GenericDetour;
use lazy_static::lazy_static;
use widestring::U16CStr;
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

type ConsoleWriteFn = extern "cdecl" fn(u32, *const wchar_t) -> u32;

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

extern "cdecl" fn detour_console_write(color: u32, message: *const wchar_t) -> u32 {
    println!("color = {} | message = {:?}", color, message);

    // Convert message to a utf8 string
    let s = unsafe { U16CStr::from_ptr_str(message).to_string_lossy() };
    println!("{}", s);

    let trampoline: ConsoleWriteFn =
        unsafe { std::mem::transmute(DETOUR_CONSOLE_WRITE.lock().unwrap().trampoline()) };
    return trampoline(color, message);
}

fn init() {
    // Give us a console window to write to
    unsafe { winapi::um::consoleapi::AllocConsole() };

    println!("Initializing...");

    // Enable hooks
    unsafe {
        DETOUR_CONSOLE_WRITE.lock().unwrap().enable().unwrap();
    }
}
