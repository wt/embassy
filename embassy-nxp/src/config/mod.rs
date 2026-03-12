#[cfg(feature = "mimxrt1011")]
#[cfg_attr(feature = "mimxrt1011", path = "./mimxrt1011.rs")]
pub(crate) mod init;

/// HAL configuration for the NXP board.
#[derive(Default)]
pub struct Config {}

#[cfg(not(feature = "_main_clock_init"))]
pub(crate) mod init {
    pub(crate) fn init_main_clock() {}
}
