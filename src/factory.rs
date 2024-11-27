#![allow(missing_docs)]

use std::{ffi::c_void, marker::PhantomData};

use vst3_com::IID;
use vst3_sys::{
    base::{
        kInvalidArgument, kResultOk, tresult, ClassCardinality, FactoryFlags, IPluginFactory,
        IPluginFactory2, PClassInfo, PClassInfo2, PFactoryInfo,
    },
    VST3,
};

use crate::{util, RawPlugin, Subcategory, VstPlugin};

/// A VST3 plugin factory.
#[VST3(implements(IPluginFactory, IPluginFactory2))]
pub struct Factory<P: VstPlugin> {
    marker: PhantomData<fn() -> P>,
}

impl<P: VstPlugin> Factory<P> {
    /// Create a new plugin factory.
    pub fn new() -> Box<Self> {
        Self::allocate(PhantomData)
    }
}

impl<P: VstPlugin> IPluginFactory for Factory<P> {
    unsafe fn get_factory_info(&self, info: *mut PFactoryInfo) -> tresult {
        if info.is_null() {
            return kInvalidArgument;
        }

        let info = &mut *info;
        let plugin_info = P::info();
        util::strcpy(&plugin_info.vendor, &mut info.vendor);
        util::strcpy(&plugin_info.url, &mut info.url);
        util::strcpy(&plugin_info.email, &mut info.email);
        info.flags = FactoryFlags::kUnicode as i32 | FactoryFlags::kComponentNonDiscardable as i32;

        kResultOk
    }

    unsafe fn count_classes(&self) -> i32 {
        1
    }

    unsafe fn get_class_info(&self, index: i32, info: *mut PClassInfo) -> tresult {
        if index != 0 || info.is_null() {
            return kInvalidArgument;
        }

        let info = &mut *info;
        let plugin_info = P::info();
        util::strcpy(&plugin_info.name, &mut info.name);
        util::strcpy("Audio Module Class", &mut info.category);
        info.cid.data = plugin_info.uuid.to_bytes_le();
        info.cardinality = ClassCardinality::kManyInstances as i32;

        kResultOk
    }

    unsafe fn create_instance(
        &self,
        cid: *const IID,
        _iid: *const IID,
        obj: *mut *mut c_void,
    ) -> tresult {
        if cid.is_null() || obj.is_null() {
            return kInvalidArgument;
        }

        let plugin_info = P::info();

        if (*cid).data != plugin_info.uuid.to_bytes_le() {
            return kInvalidArgument;
        }

        let raw_plugin = RawPlugin::<P>::new();
        *obj = Box::into_raw(raw_plugin) as *mut c_void;

        kResultOk
    }
}

impl<P: VstPlugin> IPluginFactory2 for Factory<P> {
    unsafe fn get_class_info2(&self, index: i32, info: *mut PClassInfo2) -> tresult {
        if index != 0 || info.is_null() {
            return kInvalidArgument;
        }

        let info = &mut *info;
        let plugin_info = P::info();

        let subcategories = plugin_info
            .subcategories
            .iter()
            .map(Subcategory::as_str)
            .collect::<Vec<_>>()
            .join("|");

        util::strcpy(&plugin_info.name, &mut info.name);
        util::strcpy("Audio Module Class", &mut info.category);
        util::strcpy(&plugin_info.vendor, &mut info.vendor);
        util::strcpy(&plugin_info.version, &mut info.version);
        util::strcpy("VST3 3.6.14", &mut info.sdk_version);
        util::strcpy(&subcategories, &mut info.subcategories);
        info.cid.data = plugin_info.uuid.to_bytes_le();
        info.cardinality = ClassCardinality::kManyInstances as i32;
        info.class_flags = 1 << 1; // kSimpleModeSupported

        kResultOk
    }
}
