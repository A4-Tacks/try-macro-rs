#![no_std]
#![doc = include_str!("../README.md")]

pub use try_macro_proc_macro::try_macro;
pub use try_macro_proc_macro::try_macro_block;
pub use stable_try_trait_v2::{
    Try,
    FromResidual,
    Residual,
};
