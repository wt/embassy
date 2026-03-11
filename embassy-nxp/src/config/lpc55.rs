use core::num::NonZero;
use nxp_pac as pac;

pub(crate) fn init_main_clock() {
    struct NormalModeSettings {
        m: u16,
        n: Option<NonZero<u8>>,
        p: Option<NonZero<u8>>,
        post_div2: bool,
    }

    let normal_mode_settings = NormalModeSettings {
        m: 25,
        n: NonZero::new(2),
        p: None,
        post_div2: false,
    };

    pub fn set_flash_wait_intervals_for_freq(new_freq: u32) {
        let new_wait_intervals = match new_freq {
            0..=11_000_000 => pac::syscon::vals::Flashtim::FLASHTIM0,
            ..=22_000_000 => pac::syscon::vals::Flashtim::FLASHTIM1,
            ..=33_000_000 => pac::syscon::vals::Flashtim::FLASHTIM2,
            ..=44_000_000 => pac::syscon::vals::Flashtim::FLASHTIM3,
            ..=55_000_000 => pac::syscon::vals::Flashtim::FLASHTIM4,
            ..=66_000_000 => pac::syscon::vals::Flashtim::FLASHTIM5,
            ..=84_000_000 => pac::syscon::vals::Flashtim::FLASHTIM6,
            ..=104_000_000 => pac::syscon::vals::Flashtim::FLASHTIM7,
            ..=119_000_000 => pac::syscon::vals::Flashtim::FLASHTIM8,
            ..=129_000_000 => pac::syscon::vals::Flashtim::FLASHTIM9,
            ..=144_000_000 => pac::syscon::vals::Flashtim::FLASHTIM10,
            ..=150_000_000 => pac::syscon::vals::Flashtim::FLASHTIM11,
            _ => panic!("Unsupported flash wait interval"),
        };

        // flash wait intervals
        pac::SYSCON.fmccr().modify(|m| m.set_prefen(false));
        pac::FLASH.int_status().write(|w| {
            w.set_fail(false);
            w.set_done(false);
            w.set_err(false);
            w.set_ecc_err(false);
        });
        pac::FLASH
            .dataw(0)
            .modify(|m| m.set_dataw((m.dataw() & 0xFFFFFFF0) | new_wait_intervals.to_bits() as u32));
        pac::FLASH.cmd().write(|w| w.set_cmd(0x2));
        while !pac::FLASH.int_status().read().done() {}
        pac::SYSCON.fmccr().modify(|m| {
            m.set_flashtim(new_wait_intervals);
            // m.set_prefen(pac::syscon::vals::Prefen::ENABLE);
        });
    }
    pub fn set_voltage_for_freq(new_freq: u32) {
        let vout = match new_freq {
            0..=100_000_000 => pac::pmc::vals::Vout::V_DCDC_1P100,
            ..=150_000_000 => pac::pmc::vals::Vout::V_DCDC_1P200,
            _ => unreachable!(),
        };

        // increase the voltage of the regulator
        pac::PMC.dcdc0().modify(|m| m.set_vout(vout));
    }

    fn normal_get_bandwidth_i(m: u16) -> u8 {
        // seli = floor(m/2) + 1 from 4.6.6.3.2 of the LPC55S16 User Manual
        match m {
            0 => unreachable!(),
            // 2 * floor(m/4) + 3
            i @ 1..122 => (2 * (i >> 2) + 3) as u8,
            // floor(8000/m)
            i @ 122..8000 => ((8000 + 1) / i - 1) as u8,
            8000.. => 1,
        }
    }

    fn normal_get_bandwidth_p(m: u16) -> u8 {
        // selp = floor(m/4) + 1 from 4.6.6.3.2 of the LPC55S16 User Manual
        match m >> 2 + 1 {
            0 => unreachable!(),
            i @ 1..31 => i as u8,
            31.. => 31,
        }
    }

    // use the fastest alternate source during PLL clock setup, the FRO 96 MHz clock
    pac::SYSCON
        .mainclksela()
        .modify(|m| m.set_sel(pac::syscon::vals::MainclkselaSel::ENUM_0X3));

    // The PLL will use the 12MHz FRO as its source.
    let src_freq = 12_000_000;
    let freq = src_freq * normal_mode_settings.m as u32
        / normal_mode_settings.n.map_or(1, |n| n.get()) as u32
        / match normal_mode_settings.post_div2 {
            true => 2,
            false => 1,
        } as u32
        / normal_mode_settings.p.map_or(1, |p| p.get()) as u32;

    set_voltage_for_freq(freq);
    set_flash_wait_intervals_for_freq(freq);

    pac::PMC.pdruncfg0().modify(|m| {
        m.set_pden_pll0(pac::pmc::vals::PdenPll0::POWEREDON);
        m.set_pden_pll0_sscg(pac::pmc::vals::PdenPll0Sscg::POWEREDON);
    });
    // 12MHz clock
    pac::SYSCON
        .pll0clksel()
        .write(|w| w.set_sel(pac::syscon::vals::Pll0clkselSel::ENUM_0X0));

    let mdiv = normal_mode_settings.m;

    pac::SYSCON.pll0ctrl().write(|w| {
        w.set_clken(true);
        w.set_seli(normal_get_bandwidth_i(mdiv));
        w.set_selp(normal_get_bandwidth_p(mdiv));
        // w.set_frmen(pac::syscon::vals::Frmen::ENABLE);

        // w.set_bypasspll(pac::syscon::vals::Pll0ctrlBypasspll::BYPASSED);
        match normal_mode_settings.n {
            Some(_) => w.set_bypassprediv(pac::syscon::vals::Pll0ctrlBypassprediv::BYPASSED),
            None => w.set_bypassprediv(pac::syscon::vals::Pll0ctrlBypassprediv::USED),
        }
        match normal_mode_settings.p {
            Some(_) => w.set_bypasspostdiv(pac::syscon::vals::Pll0ctrlBypasspostdiv::BYPASSED),
            None => w.set_bypasspostdiv(pac::syscon::vals::Pll0ctrlBypasspostdiv::USED),
        }
        match normal_mode_settings.post_div2 {
            true => w.set_bypasspostdiv2(pac::syscon::vals::Pll0ctrlBypasspostdiv2::USED),
            false => w.set_bypasspostdiv2(pac::syscon::vals::Pll0ctrlBypasspostdiv2::BYPASSED),
        }
        // w.set_bwdirect(pac::syscon::vals::Pll0ctrlBwdirect::DIRECT);
        // w.set_limupoff(true);
    });
    // pac::SYSCON.pll0ndec().write(|w| w.set_ndiv(8));
    match normal_mode_settings.n {
        Some(n) => pac::SYSCON.pll0ndec().write(|w| w.set_ndiv(n.get())),
        None => pac::SYSCON.pll0ndec().write(|_| {}),
    }
    match normal_mode_settings.p {
        Some(p) => pac::SYSCON.pll0pdec().write(|w| w.set_pdiv(p.get())),
        None => pac::SYSCON.pll0pdec().write(|_| {}),
    }
    pac::SYSCON.pll0sscg0().write(|w| w.set_md_lbs(0));
    pac::SYSCON.pll0sscg1().modify(|m| {
        m.set_md_mbs(true); //high bit of md
        m.set_mdiv_ext(normal_mode_settings.m);
        m.set_sel_ext(true);
    });

    pac::SYSCON
        .mainclkselb()
        .write(|w| w.set_sel(pac::syscon::vals::MainclkselbSel::ENUM_0X1));
}
