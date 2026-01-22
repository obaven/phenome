use ratatui::layout::{Constraint, Direction, Layout, Rect};
use ratatui::prelude::{Alignment, Frame};
use ratatui::widgets::{Block, Borders, Paragraph, Table};
use phenome_ports::PortSet;

mod comparison;
mod overview;
mod rows;

pub fn render(frame: &mut Frame, _area: Rect, ports: &PortSet) {
    let states = ports.bootstrap.component_states();
    let overall_text = overview::build_overall_text(ports);

    let chunks = Layout::default()
        .direction(Direction::Vertical)
        .constraints([
            Constraint::Length(8),
            Constraint::Length(6),
            Constraint::Length(8),
            Constraint::Min(0),
        ])
        .split(frame.area());

    let overall = Paragraph::new(overall_text)
        .block(
            Block::default()
                .title("Overall Status")
                .borders(Borders::ALL),
        )
        .alignment(Alignment::Left);
    frame.render_widget(overall, chunks[0]);

    let timing_rows = rows::build_timing_rows(&states);
    let timing_table = Table::new(
        timing_rows,
        [
            Constraint::Percentage(35),
            Constraint::Percentage(25),
            Constraint::Percentage(40),
        ],
    )
    .block(
        Block::default()
            .title("Timing Breakdown")
            .borders(Borders::ALL),
    );
    frame.render_widget(timing_table, chunks[1]);

    let access_rows = rows::build_access_rows(&ports.bootstrap.access_urls());
    let access_table = Table::new(
        access_rows,
        [
            Constraint::Percentage(30),
            Constraint::Percentage(50),
            Constraint::Percentage(20),
        ],
    )
    .block(Block::default().title("Access URLs").borders(Borders::ALL));
    frame.render_widget(access_table, chunks[2]);

    let hotspot_rows = rows::build_hotspot_rows(&states);
    let hotspot_table = Table::new(
        hotspot_rows,
        [
            Constraint::Percentage(50),
            Constraint::Percentage(25),
            Constraint::Percentage(25),
        ],
    )
    .block(Block::default().title("Top Hotspots").borders(Borders::ALL));
    frame.render_widget(hotspot_table, chunks[3]);
}
