use std::sync::{atomic::{AtomicI32, Ordering}, Arc};

use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::{RBox, RStr, RSlice}, sabi_trait::TD_Opaque, DynTrait};
use alphacommons::{AlphaApi};
use loader::{AddonObject_Ref, AddonObject, Logger_TO, Addon, Addon_TO, IssueResult, MainInterface_TO};

#[export_root_module]
pub fn export_addon_object() -> AddonObject_Ref {
    AddonObject {
        name: RStr::from_str("simpcnt"),
        version: RStr::from_str("0.1"),
        dependency: RSlice::from_slice(&[]),
        new: new_addon,
    }
    .leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new_addon(logger: Logger_TO<'static, RBox<()>>) -> Addon_TO<'static, RBox<()>> {
    Addon_TO::from_value(SimpleCounter { counter: Arc::new(AtomicI32::new(0)), logger }, TD_Opaque)
}

pub struct SimpleCounter {
    counter: Arc<AtomicI32>,
    logger: Logger_TO<'static, RBox<()>>
}

struct ImplAlphaApi { 
    cc: Arc<AtomicI32>,
}

impl AlphaApi for ImplAlphaApi {
    fn get_counter(&self) -> i32 {
        self.cc.load(Ordering::Relaxed)
    }

    fn set_counter(&self, v: i32) {
        self.cc.store(v, Ordering::Relaxed);
    }
}

impl Addon for SimpleCounter {
    fn on_load(&mut self, _mi: MainInterface_TO<'static, RBox<()>>) -> () {
        self.counter.store(0, Ordering::Relaxed);
        self.logger.log("SimpleCounter on_load() called!".into());
    }

    fn issue(&self) -> loader::IssueResult {
        let c = self.counter.fetch_add(1, Ordering::Relaxed);
        IssueResult {
            state: loader::Sign::NEUTRAL,
            msg: format!("Counter is {}", c + 1).into()
        }
    }

    fn get_interface(&self) -> loader::BoxedAddonInterface<'static> {
        let obj = ImplAlphaApi { cc: (Arc::clone(&self.counter)) };
        DynTrait::from_value(alphacommons::AlphaApiBox::from_value(obj, TD_Opaque))
    }
}
