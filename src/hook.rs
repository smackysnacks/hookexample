pub use std::sync::Mutex;

pub use detour::GenericDetour;

#[macro_export]
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

#[macro_export]
macro_rules! enable_hook {
    ( $name:ident ) => {
        unsafe { $name.lock().unwrap().enable().unwrap() };
    };
}

#[macro_export]
macro_rules! disable_hook {
    ( $name:ident ) => {
        unsafe { $name.lock().unwrap().disable().unwrap() };
    };
}

