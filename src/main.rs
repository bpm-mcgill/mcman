use std::io;

use mcman::installer::ServerInstaller;
use ratatui::{crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind}, layout::{Constraint, Direction, Flex, Layout, Rect, Spacing}, style::{Color, Style, Stylize}, symbols::border, text::{Line, Text}, widgets::{Block, BorderType, Borders, List, ListState, Paragraph, StatefulWidget, Widget}, DefaultTerminal, Frame};
use reqwest::Client;

struct App {
    exit: bool,
    list: ServerListWidget,

    client: Client,
    installer: ServerInstaller
}

impl App {
    pub fn new() -> Self {
        let client = Client::new();
        App {
            exit: false,
            list: ServerListWidget::new(Vec::new()),
            installer: ServerInstaller::new(client.clone()),
            client
        }
    }

    pub async fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        let versions = self.installer.get_versions().await.unwrap();
        self.list.items = versions;
        while !self.exit {
            terminal.draw(|frame| self.render(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    pub fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }
    
    pub fn render(&mut self, frame: &mut Frame) {
        let [area] = Layout::horizontal([Constraint::Percentage(65)]).flex(Flex::Center).areas(frame.area());
        let [area] = Layout::vertical([Constraint::Percentage(65)]).flex(Flex::Center).areas(area);
        frame.render_widget(&mut self.list, area);
        
        let b = Block::new().title_bottom(Line::from("mcman v0.0.12").right_aligned());
        frame.render_widget(b,frame.area());
    }

    pub fn handle_key_event(&mut self, event: KeyEvent) {
        match event.code {
            KeyCode::Char('q') => {self.exit = true},
            KeyCode::Up => {
                self.list.select_prev();
            }
            KeyCode::Down => {
                self.list.select_next();
            }
            _ => {}
        }
    }
}

struct ServerListWidget {
    items: Vec<String>,
    state: ListState
}

impl ServerListWidget {
    fn new(items: Vec<String>) -> ServerListWidget {
        ServerListWidget {
            items,
            state: ListState::default().with_selected(Some(0)), 
        }
    }
    
    fn select_prev(&mut self){
        if let Some(index) = self.state.selected() {
            let idx = index.saturating_sub(1);
            self.state.select(Some(idx));
        }
    }

    fn select_next(&mut self){
        if let Some(index) = self.state.selected() {
            let idx = index.saturating_add(1);
            self.state.select(Some(idx));
        }
    }
}

impl Widget for &mut ServerListWidget {
    fn render(self, area: ratatui::prelude::Rect, buf: &mut ratatui::prelude::Buffer)
        where
            Self: Sized {
        let title = Line::from(" Server List ".bold());
        let keybinds = Line::from(vec![
            " New Server".into(),
            "<N>".blue().bold(),
            "  Delete Server".into(),
            "<D>".blue().bold(),
            "  Select".into(),
            "<Enter> ".blue().bold(),
        ]);
        let block = Block::bordered().title(title.centered()).title_bottom(keybinds.centered()).border_set(border::ROUNDED); 
        let list = List::new(self.items.clone())
            .highlight_symbol("> ")
            .highlight_style(
                Style::new()
                .fg(Color::Black)
                .bg(Color::Gray).bold()
            )
            .block(block);
        StatefulWidget::render(list, area, buf, &mut self.state);
    }
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>>{
    let mut app = App::new();

    let mut terminal = ratatui::init();
    app.run(&mut terminal).await;
    
    ratatui::restore();
    Ok(())
}
