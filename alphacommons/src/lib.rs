use abi_stable::{sabi_trait, std_types::RBox};

#[sabi_trait]
pub trait AlphaApi : Send + Sync {
    fn get_counter(&self) -> i32;
    fn set_counter(&self, v: i32) -> ();
}

pub type AlphaApiBox = AlphaApi_TO<'static, RBox<()>>;
