use nom::{
    branch::alt,
    bytes::complete::{tag, take_until},
    character::complete::u32,
    combinator::{map, value},
    multi::separated_list1,
    IResult, Parser,
};

#[derive(Debug, Default, Clone, Copy, PartialEq)]
pub struct Cpu {
    pub load: u32,
    pub freq: u32,
}

#[derive(Debug, Default)]
pub struct Tegrastats {
    pub ram_used: u32,
    pub ram_total: u32,
    pub lfb_blocks: u32,
    pub lfb_size: u32,

    pub swap_used: u32,
    pub swap_total: u32,
    pub swap_cached: u32,

    // one entry per core, None when the core is offline
    pub cpus: Vec<Option<Cpu>>,

    // not reported on every jetson (e.g. orin)
    pub emc_freq: Option<u32>,
    pub gr3d_freq: u32,

    // sensor name and temperature in celsius, in the order tegrastats reports them
    pub temps: Vec<(String, f32)>,
}

fn cpu(input: &str) -> IResult<&str, Option<Cpu>> {
    alt((
        value(None, tag("off")),
        map((u32, tag("%@"), u32), |(load, _, freq)| Some(Cpu { load, freq })),
    ))
    .parse(input)
}

pub fn parse(input: &str) -> IResult<&str, Tegrastats> {
    // newer tegrastats versions prefix each line with a timestamp
    let (input, _) = take_until("RAM ")(input)?;
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
    let (input, cpus) = separated_list1(tag(","), cpu).parse(input)?;
    let (input, _) = tag("]")(input)?;

    // the remaining fields vary between jetson models, so pick out the known ones
    let mut emc_freq = None;
    let mut gr3d_freq = 0;
    let mut temps = Vec::new();

    let mut tokens = input.split_whitespace();
    while let Some(token) = tokens.next() {
        match token {
            "EMC_FREQ" => emc_freq = tokens.next().and_then(leading_u32),
            "GR3D_FREQ" => gr3d_freq = tokens.next().and_then(leading_u32).unwrap_or(0),
            _ => {
                if let Some((name, temp)) = token.split_once('@') {
                    if let Some(Ok(temp)) = temp.strip_suffix('C').map(str::parse::<f32>) {
                        temps.push((name.to_string(), temp));
                    }
                }
            }
        }
    }

    let tegrastats = Tegrastats {
        ram_used,
        ram_total,
        lfb_blocks,
        lfb_size,
        swap_used,
        swap_total,
        swap_cached,
        cpus,
        emc_freq,
        gr3d_freq,
        temps,
    };

    Ok(("", tegrastats))
}

// parses the number at the start of values like "27%" or "0%@1600"
fn leading_u32(s: &str) -> Option<u32> {
    let end = s.find(|c: char| !c.is_ascii_digit()).unwrap_or(s.len());
    s[..end].parse().ok()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_nano() {
        let line = "RAM 1543/3964MB (lfb 120x4MB) SWAP 0/1982MB (cached 0MB) CPU [2%@102,1%@102,off,0%@102] EMC_FREQ 3% GR3D_FREQ 5% PLL@24C CPU@26.5C PMIC@50C GPU@25.5C AO@31.5C thermal@25.75C\n";
        let (_, t) = parse(line).unwrap();
        assert_eq!(t.ram_used, 1543);
        assert_eq!(t.cpus.len(), 4);
        assert_eq!(t.cpus[0], Some(Cpu { load: 2, freq: 102 }));
        assert_eq!(t.cpus[2], None);
        assert_eq!(t.emc_freq, Some(3));
        assert_eq!(t.gr3d_freq, 5);
        assert_eq!(t.temps.len(), 6);
        assert_eq!(t.temps[1], ("CPU".to_string(), 26.5));
    }

    #[test]
    fn parses_orin() {
        let line = "10-11-2026 11:29:10 RAM 4448/7620MB (lfb 1x4MB) SWAP 4/3810MB (cached 0MB) CPU [13%@1651,11%@1651,55%@1651,23%@1651,14%@729,25%@729] GR3D_FREQ 27% cpu@52.843C iwlwifi_1@51C soc2@51.843C soc0@52.031C gpu@53.5C tj@54.062C soc1@54.062C VDD_IN 5000mW/5000mW\n";
        let (_, t) = parse(line).unwrap();
        assert_eq!(t.ram_total, 7620);
        assert_eq!(t.cpus.len(), 6);
        assert_eq!(t.cpus[5], Some(Cpu { load: 25, freq: 729 }));
        assert_eq!(t.emc_freq, None);
        assert_eq!(t.gr3d_freq, 27);
        assert_eq!(t.temps.len(), 7);
        assert_eq!(t.temps[0], ("cpu".to_string(), 52.843));
    }
}
