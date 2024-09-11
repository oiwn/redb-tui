use ratatui::{
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Style},
    widgets::{Block, Borders, List, ListItem, ListState, Paragraph},
    Frame,
};

pub fn get_layout(size: Rect) -> (Rect, Rect, Rect) {
    let main_layout = Layout::default()
        .direction(Direction::Vertical)
        .constraints([Constraint::Min(3), Constraint::Length(4)])
        .split(size);

    let top_layout = Layout::default()
        .direction(Direction::Horizontal)
        .constraints([Constraint::Percentage(30), Constraint::Percentage(70)])
        .split(main_layout[0]);

    (top_layout[0], top_layout[1], main_layout[1])
}

pub fn render_table_list(
    frame: &mut Frame,
    area: Rect,
    table_names: &[String],
    list_state: &mut ListState,
) {
    let items: Vec<ListItem> = table_names
        .iter()
        .map(|name| ListItem::new(name.as_str()))
        .collect();

    let list = List::new(items)
        .block(Block::default().title("ReDB Tables").borders(Borders::ALL))
        .style(Style::default().fg(Color::White))
        .highlight_style(Style::default().bg(Color::LightGreen).fg(Color::Black));

    frame.render_stateful_widget(list, area, list_state);
}

pub fn render_key_value_pairs(
    frame: &mut Frame,
    area: Rect,
    selected_table: &str,
    key_value_pairs: &[(String, String)],
) {
    let content = key_value_pairs
        .iter()
        .map(|(k, v)| format!("{}: {}", k, v))
        .collect::<Vec<String>>()
        .join("\n");

    let paragraph = Paragraph::new(content).block(
        Block::default()
            .title(format!("Table: {}", selected_table))
            .borders(Borders::ALL),
    );

    frame.render_widget(paragraph, area);
}

pub fn render_bottom_status(frame: &mut Frame, area: Rect, status: &str) {
    let status_widget = Paragraph::new(status)
        .block(
            Block::default()
                .title("Database Info")
                .borders(Borders::ALL),
        )
        .style(Style::default().fg(Color::Yellow));

    frame.render_widget(status_widget, area);
}
