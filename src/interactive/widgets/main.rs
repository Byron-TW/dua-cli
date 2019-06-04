use crate::{
    interactive::{
        widgets::{Entries, Footer},
        AppState, DisplayOptions,
    },
    traverse::Traversal,
};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::Widget,
};

pub struct MainWindow<'a, 'b> {
    pub traversal: &'a Traversal,
    pub display: DisplayOptions,
    pub state: &'b AppState,
}

impl<'a, 'b, 'c> Widget for MainWindow<'a, 'b> {
    fn draw(&mut self, area: Rect, buf: &mut Buffer) {
        let Self {
            traversal:
                Traversal {
                    tree,
                    entries_traversed,
                    total_bytes,
                    ..
                },
            display,
            state,
        } = self;
        let regions = Layout::default()
            .direction(Direction::Vertical)
            .constraints([Constraint::Max(256), Constraint::Length(1)].as_ref())
            .split(area);
        let (entries, footer) = (regions[0], regions[1]);
        Entries {
            tree: &tree,
            root: state.root,
            display: *display,
            sorting: state.sorting,
            selected: state.selected,
            list_start: state.entries_list_start,
        }
        .draw(entries, buf);

        Footer {
            total_bytes: *total_bytes,
            entries_traversed: *entries_traversed,
            format: display.byte_format,
            message: state.message.clone(),
        }
        .draw(footer, buf);
    }
}