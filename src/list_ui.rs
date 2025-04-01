use ratatui::{widgets::ListState, DefaultTerminal, Frame};
use crossterm::event::{self, Event};
use color_eyre::Result;

use crate::kondo::Task;

pub struct TaskWidget {
    task_list: TaskList,
    exit: bool,
}

pub struct TaskList {
    items: Vec<Task>,
    list_state: ListState,
}

pub fn run() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    loop {
        terminal.draw(render)?;
        if matches!(event::read()?, Event::Key(_)) {
            break;
        }
    }

    ratatui::restore();
    Ok(())
}

fn render(frame: &mut Frame) {
    frame.render_widget("Test", frame.area());
}
