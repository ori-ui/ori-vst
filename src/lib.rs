mod audio_layout;
mod buffer;
mod component;
mod controller;
mod editor;
mod factory;
mod param;
mod plugin;
mod processor;
mod state;
mod unit;
mod util;
mod view;

pub use audio_layout::*;
pub use buffer::*;
pub use factory::*;
pub use param::*;
pub use plugin::*;
use state::*;
use view::*;

pub fn panic_handler(info: &std::panic::PanicInfo) {
    let backtrace = std::backtrace::Backtrace::capture();

    _ = std::fs::write(
        "/home/anon/ori_panic.log",
        format!("{}\n\n{}", info, backtrace),
    );
    std::process::exit(1);
}

#[macro_export]
macro_rules! vst3 {
    ($plugin:ty) => {
        #[doc(hidden)]
        mod vst3 {
            #[no_mangle]
            unsafe extern "system" fn GetPluginFactory() -> *mut ::std::ffi::c_void {
                use super::*;

                ::std::panic::set_hook(::std::boxed::Box::new($crate::panic_handler));

                let factory = $crate::Factory::<$plugin>::new();

                ::std::boxed::Box::into_raw(factory) as *mut ::std::ffi::c_void
            }

            #[no_mangle]
            #[cfg(target_os = "linux")]
            extern "system" fn ModuleEntry(_: *mut ::std::ffi::c_void) -> bool {
                true
            }

            #[no_mangle]
            #[cfg(target_os = "linux")]
            extern "system" fn ModuleExit() -> bool {
                true
            }

            #[no_mangle]
            #[cfg(target_os = "windows")]
            extern "system" fn InitDll() -> bool {
                true
            }

            #[no_mangle]
            #[cfg(target_os = "windows")]
            extern "system" fn ExitDll() -> bool {
                true
            }

            #[no_mangle]
            #[cfg(target_os = "macos")]
            extern "system" fn bundleEntry(_: *mut ::std::ffi::c_void) -> bool {
                true
            }

            #[no_mangle]
            #[cfg(target_os = "macos")]
            extern "system" fn bundleExit() -> bool {
                true
            }
        }
    };
}
