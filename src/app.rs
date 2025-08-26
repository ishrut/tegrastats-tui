use std::process::{Command, Stdio, ChildStdout};
use std::io::{BufRead, BufReader};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Stylize, Color},
    widgets::{Gauge, Paragraph, Block, Borders},
    layout::{Layout, Constraint, Alignment},
};

use crate::parser::{self, Tegrastats};

#[derive(Debug)]
pub struct App {
    running: bool,
    tegrastats: Tegrastats,
    tegrastats_reader: BufReader<ChildStdout>,
}

impl App {
    pub fn new() -> Self {

        let mut child = Command::new("tegrastats")
            .stdout(Stdio::piped())
            .spawn()
            .expect("failed to start tegrastats, is it installed?");

        let stdout = child.stdout.take().expect("couldn't get the stdout on tegrastats");
        let reader = BufReader::new(stdout);

        Self { running: true, tegrastats: Tegrastats::default(), tegrastats_reader: reader }
    }

    pub fn update(&mut self) {
        let mut buf = String::new();
        if let Ok(_) = self.tegrastats_reader.read_line(&mut buf) {
            if let Ok((_, tegrastats)) = parser::parse(&buf) {
                self.tegrastats = tegrastats;
            }
        }
    }

    pub fn run(mut self, mut terminal: DefaultTerminal) -> Result<(), Box<dyn std::error::Error>> {
        self.running = true;
        while self.running {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_crossterm_events()?;
            self.update();
        }
        Ok(())
    }

    fn render(&mut self, frame: &mut Frame) {
        use Constraint::{Length, Min, Ratio};

        let layout = Layout::vertical([Length(3), Min(0), Length(1)]);
        let [header_area, middle_area, footer_area] = layout.areas(frame.area());

        frame.render_widget(
        Paragraph::new("Tegrastats TUI")
            .bold()
            .alignment(Alignment::Center)
            .block(Block::bordered()),
            header_area
        );

        frame.render_widget(
        Paragraph::new("Press q to quit")
            .alignment(Alignment::Center)
            .bold(),
            footer_area,
        );

        let middle_layout = Layout::vertical([Length(3), Length(3), Length(3), Length(3), Length(4), Length(3), Length(3), Length(4)]);
        let [upper_cpu_area, lower_cpu_area,  ram_area, swap_area, lfb_area, emc_area, gr3d_area, temp_area] = middle_layout.areas(middle_area);

        let upper_cpu_layout = Layout::horizontal([Ratio(1,2); 2]);
        let [cpu0_area, cpu1_area] = upper_cpu_layout.areas(upper_cpu_area);

        let cpu0_layout = Layout::horizontal([Min(0), Length(7)]);
        let [cpu0_bar_area, cpu0_freq_area] = cpu0_layout.areas(cpu0_area);

        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("cpu0 "))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.cpu0_load as u16),
            cpu0_bar_area
        );
        frame.render_widget(
            Paragraph::new(format!("\n{}MHz", self.tegrastats.cpu0_freq))
            .alignment(Alignment::Center),
            cpu0_freq_area
        );

        let cpu1_layout = Layout::horizontal([Min(0), Length(7)]);
        let [cpu1_bar_area, cpu1_freq_area] = cpu1_layout.areas(cpu1_area);
        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("cpu1 "))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.cpu1_load as u16),
            cpu1_bar_area
        );
        frame.render_widget(
            Paragraph::new(format!("\n{}MHz", self.tegrastats.cpu1_freq))
            .alignment(Alignment::Center),
            cpu1_freq_area
        );

        //
        let lower_cpu_layout = Layout::horizontal([Ratio(1,2); 2]);
        let [cpu2_area, cpu3_area] = lower_cpu_layout.areas(lower_cpu_area);
        //
        let cpu2_layout = Layout::horizontal([Min(0), Length(7)]);
        let [cpu2_bar_area, cpu2_freq_area] = cpu2_layout.areas(cpu2_area);
        //
        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("cpu2 "))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.cpu2_load as u16),
            cpu2_bar_area
        );
        frame.render_widget(
            Paragraph::new(format!("\n{}MHz", self.tegrastats.cpu2_freq))
            .alignment(Alignment::Center),
            cpu2_freq_area
        );

        let cpu3_layout = Layout::horizontal([Min(0), Length(7)]);
        let [cpu3_bar_area, cpu3_freq_area] = cpu3_layout.areas(cpu3_area);
        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("cpu3 "))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.cpu3_load as u16),
            cpu3_bar_area
        );
        frame.render_widget(
            Paragraph::new(format!("\n{}MHz", self.tegrastats.cpu3_freq))
            .alignment(Alignment::Center),
            cpu3_freq_area
        );

        let ram_layout = Layout::horizontal([Min(0), Length(20)]);
        let [ram_bar_area, ram_info_area] = ram_layout.areas(ram_area);

        let mut ram_ratio = self.tegrastats.ram_used as f64 / self.tegrastats.ram_total as f64;
        if ram_ratio.is_nan() || ram_ratio.is_infinite() {
            ram_ratio = 0.;
        }
        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("RAM"))
            .gauge_style(Color::Gray)
            .ratio(ram_ratio),
            ram_bar_area
        );
        frame.render_widget(
            Paragraph::new(format!("\n{}/{}MB", self.tegrastats.ram_used,self.tegrastats.ram_total))
            .alignment(Alignment::Center),
            ram_info_area
        );

        let swap_layout = Layout::horizontal([Min(0), Length(20)]);
        let [swap_bar_area, swap_info_area] = swap_layout.areas(swap_area);

        let mut swap_ratio = self.tegrastats.swap_used as f64 / self.tegrastats.swap_total as f64;
        if swap_ratio.is_nan() || swap_ratio.is_infinite() {
            swap_ratio = 0.;
        }
        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("SWAP"))
            .gauge_style(Color::Gray)
            .ratio(swap_ratio),
            swap_bar_area
        );
        frame.render_widget(
            Paragraph::new(format!("{}/{}MB\nCached {}", self.tegrastats.swap_used,self.tegrastats.swap_total, self.tegrastats.swap_cached))
            .alignment(Alignment::Center),
            swap_info_area
        );

        frame.render_widget(
            Paragraph::new(format!("lfb blocks {}Mb\nlfb block size {}", self.tegrastats.lfb_blocks, self.tegrastats.lfb_size))
            .alignment(Alignment::Left)
            .block(Block::bordered().title("lfb")),
            lfb_area
        );

        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("EMC"))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.emc_freq as u16),
            emc_area
        );

        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("GR3D"))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.gr3d_freq as u16),
            gr3d_area
        );

        let temp_vertical_layout = Layout::vertical([Ratio(1,2); 2]);
        let [upper_temp_area, lower_temp_area] = temp_vertical_layout.areas(temp_area);

        let temp_upper_layout = Layout::horizontal([Ratio(1, 3); 3]);
        let [pll_temp_area, cpu_temp_area, pmic_temp_area] = temp_upper_layout.areas(upper_temp_area);

        let temp_lower_layout = Layout::horizontal([Ratio(1,3); 3]);
        let [gpu_temp_area, ao_temp_area, thermal_temp_area] = temp_lower_layout.areas(lower_temp_area);

        frame.render_widget(
            Paragraph::new(format!("pll temp: {}", self.tegrastats.pll_temp))
            .alignment(Alignment::Left)
            .block(Block::bordered().title("Temp").borders(Borders::LEFT | Borders::TOP)),
            pll_temp_area
        );

        frame.render_widget(
            Paragraph::new(format!("cpu temp: {}", self.tegrastats.cpu_temp))
            .alignment(Alignment::Left)
            .block(Block::bordered().borders(Borders::TOP)),
            cpu_temp_area
        );

        frame.render_widget(
            Paragraph::new(format!("pmic temp: {}", self.tegrastats.pmic_temp))
            .alignment(Alignment::Left)
            .block(Block::bordered().borders(Borders::TOP | Borders::RIGHT)),
            pmic_temp_area
        );

        frame.render_widget(
            Paragraph::new(format!("gpu temp: {}", self.tegrastats.gpu_temp))
            .alignment(Alignment::Left)
            .block(Block::bordered().borders(Borders::BOTTOM | Borders::LEFT)),
            gpu_temp_area
        );

        frame.render_widget(
            Paragraph::new(format!("ao temp: {}", self.tegrastats.ao_temp))
            .alignment(Alignment::Left)
            .block(Block::bordered().borders(Borders::BOTTOM)),
            ao_temp_area
        );

        frame.render_widget(
            Paragraph::new(format!("thermal temp: {}", self.tegrastats.thermal_temp))
            .alignment(Alignment::Left)
            .block(Block::bordered().borders(Borders::BOTTOM | Borders::RIGHT)),
            thermal_temp_area
        );

    }

    fn handle_crossterm_events(&mut self) -> Result<(), Box<dyn std::error::Error>> {
        if event::poll(std::time::Duration::from_millis(100))? {
            match event::read()? {
                Event::Key(key) if key.kind == KeyEventKind::Press => self.on_key_event(key),
                Event::Mouse(_) => {}
                Event::Resize(_, _) => {}
                _ => {}
            }
        }
        Ok(())
    }

    fn on_key_event(&mut self, key: KeyEvent) {
        match (key.modifiers, key.code) {
            (_, KeyCode::Esc | KeyCode::Char('q'))
            | (KeyModifiers::CONTROL, KeyCode::Char('c') | KeyCode::Char('C')) => self.quit(),
            // Add other key handlers here.
            _ => {}
        }
    }

    fn quit(&mut self) {
        self.running = false;
    }
}

