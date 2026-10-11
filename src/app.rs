use std::process::{Command, Stdio, ChildStdout};
use std::io::{BufRead, BufReader};

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind, KeyModifiers};
use ratatui::{
    DefaultTerminal, Frame,
    style::{Stylize, Color},
    widgets::{Gauge, Paragraph, Block},
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

        let cpu_rows = self.tegrastats.cpus.len().div_ceil(2).max(1);
        let temp_rows = self.tegrastats.temps.len().div_ceil(3).max(1);

        let mut middle_constraints = vec![Length(3); cpu_rows];
        middle_constraints.extend([Length(3), Length(3), Length(4)]);
        if self.tegrastats.emc_freq.is_some() {
            middle_constraints.push(Length(3));
        }
        middle_constraints.extend([Length(3), Length(temp_rows as u16 + 2)]);
        let middle_areas = Layout::vertical(middle_constraints).split(middle_area);
        let (cpu_row_areas, rest) = middle_areas.split_at(cpu_rows);
        let mut rest = rest.iter().copied();
        let ram_area = rest.next().unwrap();
        let swap_area = rest.next().unwrap();
        let lfb_area = rest.next().unwrap();
        let emc_area = self.tegrastats.emc_freq.map(|_| rest.next().unwrap());
        let gr3d_area = rest.next().unwrap();
        let temp_area = rest.next().unwrap();

        for (i, cpu) in self.tegrastats.cpus.iter().enumerate() {
            let [left, right] = Layout::horizontal([Ratio(1,2); 2]).areas(cpu_row_areas[i / 2]);
            let cpu_area = if i % 2 == 0 { left } else { right };

            let cpu_layout = Layout::horizontal([Min(0), Length(7)]);
            let [cpu_bar_area, cpu_freq_area] = cpu_layout.areas(cpu_area);

            let (load, freq) = match cpu {
                Some(cpu) => (cpu.load, format!("{}MHz", cpu.freq)),
                None => (0, "off".to_string()),
            };
            frame.render_widget(
            Gauge::default()
                .block(Block::bordered().title(format!("cpu{} ", i)))
                .gauge_style(Color::Gray)
                .percent(load.min(100) as u16),
                cpu_bar_area
            );
            frame.render_widget(
                Paragraph::new(format!("\n{}", freq))
                .alignment(Alignment::Center),
                cpu_freq_area
            );
        }

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

        if let (Some(emc_freq), Some(emc_area)) = (self.tegrastats.emc_freq, emc_area) {
            frame.render_widget(
            Gauge::default()
                .block(Block::bordered().title("EMC"))
                .gauge_style(Color::Gray)
                .percent(emc_freq.min(100) as u16),
                emc_area
            );
        }

        frame.render_widget(
        Gauge::default()
            .block(Block::bordered().title("GR3D"))
            .gauge_style(Color::Gray)
            .percent(self.tegrastats.gr3d_freq.min(100) as u16),
            gr3d_area
        );

        let temp_block = Block::bordered().title("Temp");
        let temp_inner_area = temp_block.inner(temp_area);
        frame.render_widget(temp_block, temp_area);

        let temp_row_areas = Layout::vertical(vec![Length(1); temp_rows]).split(temp_inner_area);
        for (i, (name, temp)) in self.tegrastats.temps.iter().enumerate() {
            let temp_cells = Layout::horizontal([Ratio(1,3); 3]).split(temp_row_areas[i / 3]);
            frame.render_widget(
                Paragraph::new(format!("{} temp: {}", name, temp))
                .alignment(Alignment::Left),
                temp_cells[i % 3]
            );
        }
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

