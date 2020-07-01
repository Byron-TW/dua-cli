use crate::interactive::{
    fit_string_graphemes_with_ellipsis, path_of, widgets::entry_color, CursorDirection,
};
use dua::{
    traverse::{Tree, TreeIndex},
    ByteFormat,
};
use itertools::Itertools;
use std::{
    borrow::Borrow,
    collections::{btree_map::Entry, BTreeMap},
    path::PathBuf,
};
use termion::{event::Key, event::Key::*};
use tui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    widgets::{Block, Borders, Paragraph, Text, Widget},
};
use tui_react::{
    draw_text_nowrap_fn,
    util::{block_width, rect, rect::line_bound},
    List, ListProps,
};
use unicode_segmentation::UnicodeSegmentation;

pub enum MarkMode {
    Delete,
}

pub type EntryMarkMap = BTreeMap<TreeIndex, EntryMark>;
pub struct EntryMark {
    pub size: u128,
    pub path: PathBuf,
    pub index: usize,
    pub num_errors_during_deletion: usize,
    pub is_dir: bool,
}

#[derive(Default)]
pub struct MarkPane {
    selected: Option<usize>,
    marked: EntryMarkMap,
    list: List,
    has_focus: bool,
    last_sorting_index: usize,
}

pub struct MarkPaneProps {
    pub border_style: Style,
    pub format: ByteFormat,
}

impl MarkPane {
    #[cfg(test)]
    pub fn has_focus(&self) -> bool {
        self.has_focus
    }
    pub fn set_focus(&mut self, has_focus: bool) {
        self.has_focus = has_focus;
        if has_focus {
            self.selected = Some(self.marked.len().saturating_sub(1));
        } else {
            self.selected = None
        }
    }
    pub fn toggle_index(
        mut self,
        index: TreeIndex,
        tree: &Tree,
        is_dir: bool,
        toggle: bool,
    ) -> Option<Self> {
        match self.marked.entry(index) {
            Entry::Vacant(entry) => {
                if let Some(e) = tree.node_weight(index) {
                    let sorting_index = self.last_sorting_index + 1;
                    self.last_sorting_index = sorting_index;
                    entry.insert(EntryMark {
                        size: e.size,
                        path: path_of(tree, index),
                        index: sorting_index,
                        num_errors_during_deletion: 0,
                        is_dir,
                    });
                }
            }
            Entry::Occupied(entry) => {
                if toggle {
                    entry.remove();
                }
            }
        };
        if self.marked.is_empty() {
            None
        } else {
            Some(self)
        }
    }
    pub fn marked(&self) -> &EntryMarkMap {
        &self.marked
    }
    pub fn key(mut self, key: Key) -> Option<(Self, Option<MarkMode>)> {
        let action = None;
        match key {
            Ctrl('r') => return self.prepare_deletion(),
            Char('x') | Char('d') | Char(' ') => {
                return self.remove_selected().map(|s| (s, action))
            }
            Ctrl('u') | PageUp => self.change_selection(CursorDirection::PageUp),
            Char('k') | Up => self.change_selection(CursorDirection::Up),
            Char('j') | Down => self.change_selection(CursorDirection::Down),
            Ctrl('d') | PageDown => self.change_selection(CursorDirection::PageDown),
            _ => {}
        };
        Some((self, action))
    }

    pub fn iterate_deletable_items(
        mut self,
        mut delete_fn: impl FnMut(Self, TreeIndex) -> Result<Self, (Self, usize)>,
    ) -> Option<Self> {
        loop {
            match self.next_entry_for_deletion() {
                Some(entry_to_delete) => match delete_fn(self, entry_to_delete) {
                    Ok(pane) => {
                        self = pane;
                        match self.delete_entry() {
                            Some(p) => self = p,
                            None => return None,
                        }
                    }
                    Err((pane, num_errors)) => {
                        self = pane;
                        self.set_error_on_marked_item(num_errors)
                    }
                },
                None => return Some(self),
            }
        }
    }

    fn next_entry_for_deletion(&mut self) -> Option<TreeIndex> {
        match self.selected.and_then(|selected| {
            self.tree_index_by_list_position(selected)
                .and_then(|idx| self.marked.get(&idx).map(|d| (selected, idx, d)))
        }) {
            Some((position, selected_index, data)) => match data.num_errors_during_deletion {
                0 => Some(selected_index),
                _ => {
                    self.selected = match position + 1 {
                        p if p < self.marked.len() => Some(p),
                        _ => Some(self.marked.len().saturating_sub(1)),
                    };
                    self.tree_index_by_list_position(position + 1)
                }
            },
            None => None,
        }
    }
    fn delete_entry(self) -> Option<Self> {
        self.remove_selected()
    }
    fn set_error_on_marked_item(&mut self, num_errors: usize) {
        if let Some(d) = self
            .selected
            .and_then(|s| self.tree_index_by_list_position(s))
            .and_then(|p| self.marked.get_mut(&p))
        {
            d.num_errors_during_deletion = num_errors;
        }
    }
    fn prepare_deletion(mut self) -> Option<(Self, Option<MarkMode>)> {
        for entry in self.marked.values_mut() {
            entry.num_errors_during_deletion = 0;
        }
        self.selected = Some(0);
        Some((self, Some(MarkMode::Delete)))
    }
    fn remove_selected(mut self) -> Option<Self> {
        if let Some(mut selected) = self.selected {
            let idx = self.tree_index_by_list_position(selected);
            let se_len = self.marked.len();
            if let Some(idx) = idx {
                self.marked.remove(&idx);
                let new_len = se_len.saturating_sub(1);
                if new_len == 0 {
                    return None;
                }
                if new_len == selected {
                    selected = selected.saturating_sub(1);
                }
                self.selected = Some(selected);
            }
            Some(self)
        } else {
            Some(self)
        }
    }

    fn tree_index_by_list_position(&mut self, selected: usize) -> Option<TreeIndex> {
        self.marked_sorted_by_index()
            .get(selected)
            .map(|(k, _)| *k.to_owned())
    }

    fn marked_sorted_by_index(&self) -> Vec<(&TreeIndex, &EntryMark)> {
        self.marked
            .iter()
            .sorted_by_key(|(_, v)| &v.index)
            .collect()
    }

    fn change_selection(&mut self, direction: CursorDirection) {
        self.selected = self.selected.map(|selected| {
            direction
                .move_cursor(selected)
                .min(self.marked.len().saturating_sub(1))
        });
    }

    pub fn render(&mut self, props: impl Borrow<MarkPaneProps>, area: Rect, buf: &mut Buffer) {
        let MarkPaneProps {
            border_style,
            format,
        } = props.borrow();

        let marked: &_ = &self.marked;
        let title = format!(
            "Marked {} items ({}) ",
            marked.len(),
            format.display(marked.iter().map(|(_k, v)| v.size).sum::<u128>())
        );
        let selected = self.selected;
        let has_focus = self.has_focus;
        let entries = marked.values().sorted_by_key(|v| &v.index).enumerate().map(
            |(idx, v): (usize, &EntryMark)| {
                let default_style = match selected {
                    Some(selected) if idx == selected => {
                        let mut modifier = Modifier::REVERSED;
                        if has_focus {
                            modifier.insert(Modifier::BOLD);
                        }
                        Style {
                            modifier,
                            ..Default::default()
                        }
                    }
                    _ => Style::default(),
                };
                let (path, path_len) = {
                    let path = format!(
                        " {}  {}",
                        v.path.display(),
                        if v.num_errors_during_deletion != 0 {
                            format!("{} IO deletion errors", v.num_errors_during_deletion)
                        } else {
                            "".to_string()
                        }
                    );
                    let num_path_graphemes = path.graphemes(true).count();
                    match num_path_graphemes + format.total_width() {
                        n if n > area.width as usize => {
                            let desired_size = num_path_graphemes - (n - area.width as usize);
                            fit_string_graphemes_with_ellipsis(
                                path,
                                num_path_graphemes,
                                desired_size,
                            )
                        }
                        _ => (path, num_path_graphemes),
                    }
                };
                let fg_path = entry_color(Color::Reset, !v.is_dir, true);
                let path = Text::Styled(
                    path.into(),
                    Style {
                        fg: fg_path,
                        ..default_style
                    },
                );
                let bytes = Text::Styled(
                    format!(
                        "{:>byte_column_width$} ",
                        format.display(v.size).to_string(), // we would have to impl alignment/padding ourselves otherwise...
                        byte_column_width = format.width()
                    )
                    .into(),
                    Style {
                        fg: Color::Green,
                        ..default_style
                    },
                );
                let spacer = Text::Styled(
                    format!(
                        "{:-space$}",
                        "",
                        space = (area.width as usize)
                            .saturating_sub(path_len)
                            .saturating_sub(format.total_width())
                    )
                    .into(),
                    Style {
                        fg: fg_path,
                        ..default_style
                    },
                );
                vec![path, spacer, bytes]
            },
        );

        let entry_in_view = match self.selected {
            Some(s) => Some(s),
            None => {
                self.list.offset = 0;
                Some(marked.len().saturating_sub(1))
            }
        };
        let block = Block::default()
            .title(&title)
            .border_style(*border_style)
            .borders(Borders::ALL);

        let inner_area = block.inner(area);
        block.render(area, buf);

        let list_area = if self.has_focus {
            let (help_line_area, list_area) = {
                let help_at_bottom =
                    selected.unwrap_or(0) >= inner_area.height.saturating_sub(1) as usize / 2;
                let constraints = {
                    let mut c = vec![Constraint::Length(1), Constraint::Max(256)];
                    if help_at_bottom {
                        c.reverse();
                    }
                    c
                };
                let regions = Layout::default()
                    .direction(Direction::Vertical)
                    .constraints(constraints)
                    .split(inner_area);

                match help_at_bottom {
                    true => (regions[1], regions[0]),
                    false => (regions[0], regions[1]),
                }
            };

            let default_style = Style {
                fg: Color::Black,
                bg: Color::Yellow,
                modifier: Modifier::BOLD,
            };
            Paragraph::new(
                [
                    Text::Styled(
                        " Ctrl + r".into(),
                        Style {
                            fg: Color::LightRed,
                            modifier: default_style.modifier | Modifier::RAPID_BLINK,
                            ..default_style
                        },
                    ),
                    Text::Styled(
                        " deletes listed entries from disk without prompt".into(),
                        default_style,
                    ),
                ]
                .iter(),
            )
            .style(default_style)
            .render(help_line_area, buf);
            list_area
        } else {
            inner_area
        };

        let props = ListProps {
            block: None,
            entry_in_view,
        };
        self.list.render(props, entries, list_area, buf);

        if has_focus {
            let help_text = " . = o|.. = u ── ⇊ = CTRL+d|↓ = j|⇈ = CTRL+u|↑ = k ";
            let help_text_block_width = block_width(help_text);
            let bound = Rect {
                width: area.width.saturating_sub(1),
                ..area
            };
            if block_width(&title) + help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    help_text,
                    |_, _, _| Style::default(),
                );
            }
            let bound = line_bound(bound, bound.height.saturating_sub(1) as usize);
            let help_text = " mark-toggle = space|d";
            let help_text_block_width = block_width(help_text);
            if help_text_block_width <= bound.width {
                draw_text_nowrap_fn(
                    rect::snap_to_right(bound, help_text_block_width),
                    buf,
                    help_text,
                    |_, _, _| Style::default(),
                );
            }
        }
    }
}
