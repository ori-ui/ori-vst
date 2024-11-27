#![warn(missing_docs)]

//! # Ori VST
//! Ori VST is a framework for building VST3 plugins in Rust with graphical user interfaces.

mod audio_layout;
mod buffer;
mod component;
mod controller;
mod editor;
mod factory;
mod float;
mod param;
mod plugin;
mod processor;
mod state;
mod unit;
mod util;
mod view;

#[cfg(target_os = "linux")]
mod x11;

pub use ori::*;

pub use audio_layout::*;
pub use buffer::*;
pub use factory::*;
pub use float::*;
pub use param::*;
pub use plugin::*;
use state::*;
use view::*;

pub use uuid::Uuid;

pub use ori_vst_macro::uuid;

#[doc(hidden)]
pub fn install_log() {
    let layer = tracing_subscriber::fmt();

    let _ = ori::log::subscriber::set_global_default(layer.finish());
}

/// Macro for exporting a [`VstPlugin`] and generating the necessary boilerplate.
#[macro_export]
macro_rules! export {
    ($plugin:ty) => {
        #[doc(hidden)]
        const _: () = {
            #[no_mangle]
            unsafe extern "system" fn GetPluginFactory() -> *mut ::std::ffi::c_void {
                let factory = $crate::Factory::<$plugin>::new();

                ::std::boxed::Box::into_raw(factory) as *mut ::std::ffi::c_void
            }

            #[no_mangle]
            #[cfg(target_os = "linux")]
            extern "system" fn ModuleEntry(_: *mut ::std::ffi::c_void) -> bool {
                $crate::install_log();

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
                $crate::install_log();

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
                $crate::install_log();

                true
            }

            #[no_mangle]
            #[cfg(target_os = "macos")]
            extern "system" fn bundleExit() -> bool {
                true
            }
        };
    };
}

pub mod prelude {
    //! A prelude for convenience.

    pub use crate::{
        Activate, AudioLayout, AudioPort, Bool, Buffer, BufferLayout, Float, Info, Param,
        ParamFlags, Params, Process, Subcategory, Unit, VstPlugin,
    };

    pub use ori_vst_macro::uuid;

    pub use ori::prelude::*;
}
