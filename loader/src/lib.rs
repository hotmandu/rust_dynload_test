
use abi_stable::{
    declare_root_module_statics,
    library::RootModule,
    package_version_strings, sabi_trait,
    sabi_types::VersionStrings,
    std_types::{RBox, RStr, RString, RSlice, ROption},
    StableAbi, DynTrait,
};

#[sabi_trait]
pub trait Logger {
    fn log(&self, msg: RStr);
}

#[repr(C)]
#[derive(StableAbi)]
#[derive(Debug, Clone)]
pub enum Sign {
    POSITIVE,
    NEUTRAL,
    NEGATIVE,
}

#[repr(C)]
#[derive(StableAbi)]
pub struct IssueResult {
    pub state: Sign,
    pub msg: RString,
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(impl_InterfaceType())]
pub struct AddonInterface;

pub type BoxedAddonInterface<'borr> = DynTrait<'borr, RBox<()>, AddonInterface>;

#[sabi_trait]
pub trait MainInterface {
    fn get_interface_of(&self, addon_name: RStr) -> ROption<BoxedAddonInterface<'static>>;
}

#[sabi_trait]
pub trait Addon {
    fn on_load(&mut self, interface: MainInterface_TO<'static, RBox<()>>) -> ();

    fn issue(&self) -> IssueResult;
    fn get_interface(&self) -> BoxedAddonInterface<'static>;
}

#[repr(C)]
#[derive(StableAbi)]
#[sabi(kind(Prefix))]
pub struct AddonObject {
    pub name: RStr<'static>,
    pub version: RStr<'static>,
    
    pub dependency: RSlice<'static, RStr<'static>>,

    #[sabi(last_prefix_field)]
    pub new: extern "C" fn(Logger_TO<'static, RBox<()>>) -> Addon_TO<'static, RBox<()>>,
}

impl RootModule for AddonObject_Ref {
    // The name of the dynamic library
    const BASE_NAME: &'static str = "addon";
    // The name of the library for logging and similars
    const NAME: &'static str = "addon";
    // The version of this plugin's crate
    const VERSION_STRINGS: VersionStrings = package_version_strings!();

    declare_root_module_statics! {AddonObject_Ref}
}

