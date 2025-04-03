use color_eyre::{owo_colors::OwoColorize, Result};
use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    layout::Alignment::Center, prelude::{Buffer, Rect}, style::{
        palette::tailwind::SLATE, Color, Modifier, Style, Stylize
    }, symbols, text::{Line, Text}, widgets::{
        Block, Borders, HighlightSpacing, List, ListItem, ListState, Paragraph, StatefulWidget, Widget
    }, DefaultTerminal, Frame
};
use sqlx::SqlitePool;

use crate::{database::list_all, kondo::Task};

/*
$space-cadet: rgba(46, 41, 78, 1);
$fairy-tale: rgba(239, 188, 213, 1);
$lilac: rgba(190, 151, 198, 1);
$amethyst: rgba(134, 97, 193, 1);
$charcoal: rgba(75, 82, 103, 1);

/* SCSS RGB */
$blue-munsell: rgba(87, 143, 158, 1);
$feldgrau: rgba(92, 110, 91, 1);
$midnight-green: rgba(13, 51, 56, 1);
$sea-green: rgba(77, 143, 86, 1);
$british-racing-green: rgba(18, 69, 48, 1);
*/

const HEADER_BG: Color = Color::Rgb(13, 51, 56);
const HEADER_FG: Color = Color::Rgb(77, 143, 86);

const LIST_BG: Color = Color::Rgb(13, 51, 56); //Color::Rgb(75, 82, 103);
const LIST_ITEM_HEADER: Color = Color::Rgb(77, 143, 86);
const LIST_ITEM_BODY: Color = Color::Rgb(77, 143, 86);
const LIST_ITEM_SEPARATOR: Color = Color::Rgb(77, 143, 86);
const LIST_ITEM_TEXT: Color = Color::Rgb(92, 110, 91);

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
            .title(" [Kondo Tasks] ")
            .borders(Borders::TOP)
            .border_set(symbols::border::PLAIN)
            .border_style(Style::new().fg(HEADER_FG).bg(HEADER_BG))
            .title_alignment(Center)
            .bg(LIST_BG);

        let items: Vec<ListItem> = self
            .task_list
            .items
            .iter()
            .map(|item| ListItem::from(item))
            .collect();

        let list = List::new(items)
            .block(block)
            .bg(LIST_BG)
            .highlight_style(LIST_ITEM_BODY)
            .highlight_symbol(" > ")
            .highlight_spacing(HighlightSpacing::Always);

        StatefulWidget::render(list, area, buf, &mut self.task_list.list_state);
    }

    fn run(mut self, mut terminal: DefaultTerminal) -> Result<()> {
        while !self.exit {

            terminal.draw(|frame| {
                let size = frame.size(); // Get full terminal size

                    let block = Block::default()
                        .title("Full Screen Block")
                        .borders(Borders::NONE)
                        .style(Style::default().bg(LIST_BG));

                    frame.render_widget(block, size);

                frame.render_widget(&mut self, frame.area());
            })?;
            if let Event::Key(key) = event::read()? {
                self.handle_key(key);
            };
        }
        Ok(())
    }
}

impl From<&Task> for ListItem<'_> {
    fn from(value: &Task) -> Self {

        let multiline: Vec<Line<'_>> = value
            .content
            .split('\n')
            .map( |s| -> Line<'_> { Line::styled(s.to_string(), LIST_ITEM_TEXT) })
            .collect();

        let lines = [
            Line::styled(
                value.deadline.format("%Y-%m-%d").to_string(),
                LIST_ITEM_TEXT,
            ).add_modifier(Modifier::BOLD),
            Line::styled(
                "---------------------".to_string(),
                LIST_ITEM_SEPARATOR,
            )
        ];

        let items: Vec<Line<'_>> = lines.to_vec().iter().chain(multiline.iter()).cloned().collect();

        let text = Text::from(items);
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
