use nom::{
    bytes::complete::tag,
    character::complete::u32,
    number::complete::float,
    IResult,
};

#[derive(Debug, Default)]
pub struct Tegrastats {
    pub ram_used: u32,
    pub ram_total: u32,
    pub lfb_blocks: u32,
    pub lfb_size: u32,

    pub swap_used: u32,
    pub swap_total: u32,
    pub swap_cached: u32,

    pub cpu0_load: u32,
    pub cpu0_freq: u32,
    pub cpu1_load: u32,
    pub cpu1_freq: u32,
    pub cpu2_load: u32,
    pub cpu2_freq: u32,
    pub cpu3_load: u32,
    pub cpu3_freq: u32,

    pub emc_freq: u32,
    pub gr3d_freq: u32,

    pub pll_temp: f32,
    pub cpu_temp: f32,
    pub pmic_temp: f32,
    pub gpu_temp: f32,
    pub ao_temp: f32,
    pub thermal_temp: f32,
}

pub fn parse(input: &str) -> IResult<&str, Tegrastats> {
    let (input, _) = tag("RAM ")(input)?;
    let (input, ram_used) = u32(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, ram_total) = u32(input)?;
    let (input, _) = tag("MB (lfb ")(input)?;
    let (input, lfb_blocks) = u32(input)?;
    let (input, _) = tag("x")(input)?;
    let (input, lfb_size) = u32(input)?;
    let (input, _) = tag("MB) SWAP ")(input)?;
    let (input, swap_used) = u32(input)?;
    let (input, _) = tag("/")(input)?;
    let (input, swap_total) = u32(input)?;
    let (input, _) = tag("MB (cached ")(input)?;
    let (input, swap_cached) = u32(input)?;
    let (input, _) = tag("MB) CPU [")(input)?;

    let (input, cpu0_load) = u32(input)?;
    let (input, _) = tag("%@")(input)?;
    let (input, cpu0_freq) = u32(input)?;
    let (input, _) = tag(",")(input)?;

    let (input, cpu1_load) = u32(input)?;
    let (input, _) = tag("%@")(input)?;
    let (input, cpu1_freq) = u32(input)?;
    let (input, _) = tag(",")(input)?;

    let (input, cpu2_load) = u32(input)?;
    let (input, _) = tag("%@")(input)?;
    let (input, cpu2_freq) = u32(input)?;
    let (input, _) = tag(",")(input)?;

    let (input, cpu3_load) = u32(input)?;
    let (input, _) = tag("%@")(input)?;
    let (input, cpu3_freq) = u32(input)?;
    let (input, _) = tag("] EMC_FREQ ")(input)?;

    let (input, emc_freq) = u32(input)?;
    let (input, _) = tag("% GR3D_FREQ ")(input)?;

    let (input, gr3d_freq) = u32(input)?;
    let (input, _) = tag("% PLL@")(input)?;

    let (input, pll_temp) = float(input)?;
    let (input, _) = tag("C CPU@")(input)?;
    let (input, cpu_temp) = float(input)?;

    let (input, _) = tag("C PMIC@")(input)?;
    let (input, pmic_temp) = float(input)?;

    let (input, _) = tag("C GPU@")(input)?;
    let (input, gpu_temp) = float(input)?;

    let (input, _) = tag("C AO@")(input)?;
    let (input, ao_temp) = float(input)?;

    let (input, _) = tag("C thermal@")(input)?;
    let (input, thermal_temp) = float(input)?;
    let (input, _) = tag("C")(input)?;

    let tegrastats = Tegrastats {
        ram_used,
        ram_total,
        lfb_blocks,
        lfb_size,
        swap_used,
        swap_total,
        swap_cached,
        cpu0_load,
        cpu0_freq,
        cpu1_load,
        cpu1_freq,
        cpu2_load,
        cpu2_freq,
        cpu3_load,
        cpu3_freq,
        emc_freq,
        gr3d_freq,
        pll_temp,
        cpu_temp,
        pmic_temp,
        gpu_temp,
        ao_temp,
        thermal_temp,
    };

    Ok((input, tegrastats))
}

