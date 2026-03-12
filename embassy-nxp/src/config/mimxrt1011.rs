use nxp_pac as pac;

pub(crate) fn init_main_clock() {
    // The RT1010 Reference manual states that core clock root must be switched before
    // reprogramming PLL2.
    pac::CCM.cbcdr().modify(|w| {
        w.set_periph_clk_sel(pac::ccm::vals::PeriphClkSel::PERIPH_CLK_SEL_1);
    });

    while matches!(
        pac::CCM.cdhipr().read().periph_clk_sel_busy(),
        pac::ccm::vals::PeriphClkSelBusy::PERIPH_CLK_SEL_BUSY_1
    ) {}

    info!("Core clock root switched");

    // 480 * 18 / 24 = 360
    pac::CCM_ANALOG.pfd_480().modify(|x| x.set_pfd2_frac(12));

    //480*18/24(pfd0)/4
    pac::CCM_ANALOG.pfd_480().modify(|x| x.set_pfd0_frac(24));
    pac::CCM.cscmr1().modify(|x| x.set_flexspi_podf(3.into()));

    // CPU Core
    pac::CCM_ANALOG.pfd_528().modify(|x| x.set_pfd3_frac(18));
    cortex_m::asm::delay(500_000);

    // Clock core clock with PLL 2.
    pac::CCM
        .cbcdr()
        .modify(|x| x.set_periph_clk_sel(pac::ccm::vals::PeriphClkSel::PERIPH_CLK_SEL_0));
    // false

    while matches!(
        pac::CCM.cdhipr().read().periph_clk_sel_busy(),
        pac::ccm::vals::PeriphClkSelBusy::PERIPH_CLK_SEL_BUSY_1
    ) {}

    pac::CCM
        .cbcmr()
        .write(|v| v.set_pre_periph_clk_sel(pac::ccm::vals::PrePeriphClkSel::PRE_PERIPH_CLK_SEL_0));

    // TODO: Some for USB PLLs

    // DCDC clock?
    pac::CCM.ccgr6().modify(|v| v.set_cg0(1));
}
