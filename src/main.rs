use color_eyre::Result;
use crossterm::event::{self, Event};
use ratatui::{
    DefaultTerminal, Frame
};

fn main() -> Result<()>{
    let terminal = ratatui::init();
    let app_result = run(terminal);
    ratatui::restore();
    app_result
}

fn run(mut terminal: DefaultTerminal) -> Result<()> {
    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)){
            break Ok(());
        }
    }
}

fn render(frame: &mut Frame) {
    frame.render_widget("Yart", frame.area());
}
