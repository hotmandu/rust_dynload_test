use std::{thread::{self, JoinHandle}, cell::Cell, sync::mpsc::{self, Sender}, time::Duration};

use abi_stable::{export_root_module, prefix_type::PrefixTypeTrait, sabi_extern_fn, std_types::{RBox, RStr, RSlice}, sabi_trait::TD_Opaque, DynTrait};
use loader::{AddonObject_Ref, AddonObject, Logger_TO, Addon, Addon_TO, IssueResult, BoxedAddonInterface, MainInterface_TO};

use alphacommons::{AlphaApiBox};

static DEPENDENCY: [RStr<'static>; 1] = [RStr::from_str("simpcnt")];

#[export_root_module]
pub fn export_addon_object() -> AddonObject_Ref {
    AddonObject {
        name: RStr::from_str("timer"),
        version: RStr::from_str("0.1"),
        dependency: RSlice::from_slice(&DEPENDENCY),
        new: new_addon,
    }
    .leak_into_prefix()
}

#[sabi_extern_fn]
pub fn new_addon(logger: Logger_TO<'static, RBox<()>>) -> Addon_TO<'static, RBox<()>> {
    Addon_TO::from_value(TimerAddon { logger, alpha_interface: None.into(), thread: None.into() }, TD_Opaque)
}

pub struct TimerAddon {
    logger: Logger_TO<'static, RBox<()>>,
    alpha_interface: Cell<Option<RBox<AlphaApiBox>>>,

    thread: Cell<Option<(JoinHandle<RBox<AlphaApiBox>>, Sender<()>)>>,
}

impl Addon for TimerAddon {
    fn on_load(&mut self, mi: MainInterface_TO<'static, RBox<()>>) -> () {
        self.logger.log("TimerAddon on_load() called!".into());
        self.alpha_interface = Cell::new(
            mi.get_interface_of("simpcnt".into())
            .into_option()
            .map(|v| unsafe { v.unchecked_downcast_into::<AlphaApiBox>() } )
        );
    }

    fn issue(&self) -> loader::IssueResult {
        let inner = self.thread.take();
        if let Some((handle, send)) = inner {
            send.send(()).unwrap();
            let rebr = handle.join().unwrap();
            self.alpha_interface.replace(Some(rebr));
            self.thread.replace(None);
            IssueResult {
                state: loader::Sign::NEGATIVE,
                msg: format!("Turned OFF timer").into()
            }
        } else {
            let ai = self.alpha_interface.take();
            match ai {
                Some(rb) => {
                    let (send, recv) = mpsc::channel::<()>();
                    let rebr = rb;
                    let logger = self.logger.clone();
                    let ssp = thread::spawn(move || {
                        loop {
                            if let Ok(_) = recv.recv_timeout(Duration::from_secs(3)) {
                                // Interrupted
                                break;
                            }
                            let before = rebr.get_counter();
                            rebr.set_counter(before + 1);
                            logger.log("Increased counter!".into());
                        }
                        rebr
                    });
                    self.thread.replace(Some((ssp, send)));
                    IssueResult {
                        state: loader::Sign::POSITIVE,
                        msg: format!("Turned ON timer").into()
                    }
                },
                None => {
                    self.logger.log("Simpcnt module not found".into());
                    IssueResult {
                        state: loader::Sign::NEUTRAL,
                        msg: format!("").into()
                    }
                }
            }
        }
    }

    fn get_interface(&self) -> BoxedAddonInterface<'static> {
        DynTrait::from_value(())
    }
}
