use color_eyre::Result;
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    prelude::{Buffer, Rect}, style::{
        palette::tailwind::{BLUE, SLATE}, Style, Stylize
    }, symbols, text::{Line, Text}, widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, StatefulWidget, Widget,
    }, DefaultTerminal, Frame
};
use sqlx::SqlitePool;

use crate::{database::list_all, kondo::Task};

pub struct TaskList {
    items: Vec<Task>,
    list_state: ListState,
}

impl TaskList {
    fn new(items: Vec<Task>) -> Self {
        Self {
            items,
            list_state: ListState::default(),
        }
    }
}

pub struct TaskWidget {
    task_list: TaskList,
    exit: bool,
}

impl TaskWidget {
    async fn new(pool: &SqlitePool) -> Self {
        let task_list = TaskList::new(list_all(pool).await.expect("Can't load tasks from db"));
        let exit = false;
        TaskWidget { task_list, exit }
    }

    fn handle_key(&mut self, key: KeyEvent) {
        if key.kind != KeyEventKind::Press {
            return;
        }

        match key.code {
            KeyCode::Down => self.scroll_down(),
            KeyCode::Up => self.scroll_up(),
            KeyCode::Char(' ') => self.toggle(),
            KeyCode::Esc => self.exit = true,
            _ => {}
        }
    }

    fn scroll_down(&mut self) {
        self.task_list.list_state.select_next();
    }
    fn scroll_up(&mut self) {
        self.task_list.list_state.select_previous();
    }
    fn toggle(&mut self) {
        if let Some(task) = self.task_list.list_state.selected() {}
    }

    fn render_list(&mut self, area: Rect, buf: &mut Buffer) {
        let block = Block::new()
            .title("Kondo Tasks")
            .borders(Borders::TOP)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(SLATE.c100).bg(BLUE.c800))
            .bg(SLATE.c700);

        let items: Vec<ListItem> = self
            .task_list
            .items
            .iter()
            .map(|item| ListItem::from(item))
            .collect();

        let list = List::new(items)
            .block(block)
            .highlight_style(SLATE.c500)
            .highlight_symbol(">")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.task_list.list_state);
    }
    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {
            terminal.draw(|frame| frame.render_widget(&mut self, frame.area()))?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }
}

impl From<&Task> for ListItem<'_> {
    fn from(value: &Task) -> Self {

        let lines = [
            Line::styled(
                value.deadline.format("%Y-%m-%d").to_string(),
                SLATE.c200,
            ),
            Line::styled(
                "---------------------".to_string(),
                SLATE.c50,
            ),
            Line::styled(
                value.content.to_string(),
                SLATE.c200,
            )
        ];
        let text = Text::from(lines.to_vec());
        ListItem::new(text)
    }
}

impl Widget for &mut TaskWidget {
    fn render(self, area: Rect, buf: &mut Buffer)
    where
        Self: Sized,
    {
        self.render_list(area, buf);
    }
}

pub async fn run(pool: &SqlitePool) -> Result<()> {
    color_eyre::install()?;
    let terminal = ratatui::init();

    _ = TaskWidget::new(pool).await.run(terminal);

    ratatui::restore();
    Ok(())
}
