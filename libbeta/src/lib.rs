use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::{RBox, RStr, RSlice}, sabi_trait::TD_Opaque, DynTrait};
use loader::{AddonObject_Ref, AddonObject, Logger_TO, Addon, Addon_TO, IssueResult, BoxedAddonInterface, MainInterface_TO};

use alphacommons::{AlphaApiBox};

static DEPENDENCY: [RStr<'static>; 1] = [RStr::from_str("simpcnt")];

#[export_root_module]
pub fn export_addon_object() -> AddonObject_Ref {
    AddonObject {
        name: RStr::from_str("reduce"),
        version: RStr::from_str("0.1"),
        dependency: RSlice::from_slice(&DEPENDENCY),
        new: new_addon,
    }
    .leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new_addon(logger: Logger_TO<'static, RBox<()>>) -> Addon_TO<'static, RBox<()>> {
    Addon_TO::from_value(ReduceCounter { logger, alpha_interface: None }, TD_Opaque)
}

pub struct ReduceCounter {
    logger: Logger_TO<'static, RBox<()>>,
    alpha_interface: Option<RBox<AlphaApiBox>>
}

impl Addon for ReduceCounter {
    fn on_load(&mut self, mi: MainInterface_TO<'static, RBox<()>>) -> () {
        self.logger.log("ReduceCounter on_load() called!".into());
        self.alpha_interface = mi.get_interface_of("simpcnt".into()).into_option().map(|v| unsafe { v.unchecked_downcast_into::<AlphaApiBox>() } );
    }

    fn issue(&self) -> loader::IssueResult {
        match &self.alpha_interface {
            Some(ai) => {
                let before = ai.get_counter();
                ai.set_counter(before - 10);
                self.logger.log(format!("Reduced Counter by 10! ({} -> {})", before, before - 10).as_str().into());
                IssueResult {
                    state: loader::Sign::POSITIVE,
                    msg: format!("").into()
                }
            },
            None => {
                self.logger.log("Simpcnt module not found".into());
                IssueResult {
                    state: loader::Sign::NEGATIVE,
                    msg: format!("").into()
                }
            }
        }
    }

    fn get_interface(&self) -> BoxedAddonInterface<'static> {
        DynTrait::from_value(())
    }
}
