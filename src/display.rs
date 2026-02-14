use crate::sidebar::Sidebar;
use crate::welcome::WelcomeScreen;
use crossterm::style::Color;
use crossterm::{cursor, execute, queue, style, terminal};
use std::io;
use std::io::{BufWriter, Write};

pub struct Display {
    pub file_matrix: Vec<Vec<char>>,
    pub columns: u16,
    pub rows: u16,
    pub initial_row: u16,
    pub initial_column: u16,
    mode: String,
    modified: bool,
    cursor_line: u16,
    cursor_column: u16,
    file_size: usize,
    filename: String,
    sidebar_width: u16,
    tab_names: Vec<(String, bool, bool)>,
    show_welcome: bool,
    show_cursor: bool,
}

impl Display {
    pub fn new() -> Display {
        let (columns, rows) = terminal::size().unwrap();

        Display {
            file_matrix: vec![vec![]],
            columns,
            rows,
            initial_row: 0,
            initial_column: 0,
            mode: String::from("NORMAL"),
            modified: false,
            cursor_line: 1,
            cursor_column: 1,
            file_size: 1,
            filename: String::new(),
            sidebar_width: 0,
            tab_names: vec![],
            show_welcome: false,
            show_cursor: true,
        }
    }

    pub fn set_welcome(&mut self, show: bool) {
        self.show_welcome = show;
    }

    pub fn set_sidebar_width(&mut self, width: u16) {
        self.sidebar_width = width;
    }

    pub fn set_tab_names(&mut self, tabs: Vec<(String, bool, bool)>) {
        self.tab_names = tabs;
    }

    pub fn set_filename(&mut self, name: String) {
        self.filename = name;
    }

    pub fn set_show_cursor(&mut self, show: bool) {
        self.show_cursor = show;
    }

    fn content_start_col(&self) -> u16 {
        self.sidebar_width
    }

    fn content_width(&self) -> u16 {
        self.columns.saturating_sub(self.sidebar_width)
    }

    /// Write a full row span with a single color pair using queue! for performance.
    fn write_span(
        writer: &mut BufWriter<io::Stdout>,
        col: u16,
        row: u16,
        fg: Color,
        bg: Color,
        text: &str,
    ) {
        queue!(
            writer,
            cursor::MoveTo(col, row),
            style::SetForegroundColor(fg),
            style::SetBackgroundColor(bg),
            style::Print(text),
        )
        .unwrap();
    }

    pub fn show_display(&self, sidebar: Option<&mut Sidebar>, search_query: Option<&str>) {
        let (last_col, last_row) = cursor::position().unwrap();
        let mut writer = BufWriter::with_capacity(64 * 1024, io::stdout());

        queue!(writer, cursor::Hide).unwrap();

        let content_start = self.content_start_col();
        let content_w = self.content_width();

        // --- Draw sidebar if visible ---
        if let Some(sidebar) = sidebar {
            if sidebar.visible {
                self.render_sidebar(&mut writer, sidebar);
            }
        }

        if self.show_welcome {
            let welcome = WelcomeScreen::render(content_w, self.rows);
            for (row_idx, row) in welcome.iter().enumerate() {
                // Build spans of same color
                let mut col_idx = 0;
                while col_idx < row.len() {
                    let screen_col = content_start + col_idx as u16;
                    if screen_col >= self.columns {
                        break;
                    }
                    let fg = row[col_idx].fg_color;
                    let bg = row[col_idx].bg_color;
                    let mut span = String::new();
                    while col_idx < row.len()
                        && row[col_idx].fg_color == fg
                        && row[col_idx].bg_color == bg
                    {
                        span.push(row[col_idx].character);
                        col_idx += 1;
                    }
                    Self::write_span(&mut writer, screen_col, row_idx as u16, fg, bg, &span);
                }
            }
            queue!(writer, style::ResetColor).unwrap();
            if self.show_cursor {
                queue!(writer, cursor::Show).unwrap();
            }
            writer.flush().unwrap();
            execute!(io::stdout(), cursor::MoveTo(last_col, last_row)).unwrap();
            return;
        }

        // --- Tab bar (row 0) ---
        self.render_tab_bar(&mut writer, content_start, content_w);

        // --- Content area (rows 1 to rows-2) ---
        let content_rows = self.rows.saturating_sub(2);
        let content_start_row: u16 = 1;

        let mut file_matrix_row_start = self.initial_row;
        let mut file_matrix_row_end = file_matrix_row_start + content_rows;

        if file_matrix_row_end as usize > self.file_matrix.len() {
            let overflow = file_matrix_row_end as usize - self.file_matrix.len();
            file_matrix_row_start = file_matrix_row_start.saturating_sub(overflow as u16);
            file_matrix_row_end =
                (file_matrix_row_start + content_rows).min(self.file_matrix.len() as u16);
        }

        let row_lines_length = self.offset_lines_number();
        let row_lines = self.offset_lines(&file_matrix_row_start, &file_matrix_row_end);

        let bg_content = Color::Rgb {
            r: 15,
            g: 18,
            b: 15,
        };
        let bg_line_nr = Color::Rgb {
            r: 10,
            g: 12,
            b: 10,
        };
        let fg_text = Color::Rgb {
            r: 200,
            g: 200,
            b: 200,
        };
        let fg_line_nr = Color::Rgb {
            r: 100,
            g: 100,
            b: 100,
        };

        let search_chars: Vec<char> = search_query.unwrap_or("").chars().collect();
        let search_len = search_chars.len();
        let fg_match = Color::Rgb {
            r: 255,
            g: 200,
            b: 50,
        };
        let bg_match = Color::Rgb {
            r: 80,
            g: 60,
            b: 10,
        };

        for i in 0..content_rows {
            let screen_row = content_start_row + i;
            let file_row_idx = (file_matrix_row_start + i) as usize;

            // 1) Line number part — single span
            let line_nr_str: String = if (i as usize) < row_lines.len() {
                row_lines[i as usize].iter().collect()
            } else {
                " ".repeat(row_lines_length)
            };
            Self::write_span(
                &mut writer,
                content_start,
                screen_row,
                fg_line_nr,
                bg_line_nr,
                &line_nr_str,
            );

            // 2) Content part — build color spans
            let text_start_col = content_start + row_lines_length as u16;
            let text_width = content_w.saturating_sub(row_lines_length as u16) as usize;

            if file_row_idx < self.file_matrix.len() {
                let line = &self.file_matrix[file_row_idx];
                let mut col = 0;
                while col < text_width {
                    let file_col = self.initial_column as usize + col;
                    let ch = line.get(file_col).copied().unwrap_or(' ');

                    let is_match = if search_len > 0 {
                        self.is_search_match(line, file_col, &search_chars)
                    } else {
                        false
                    };

                    let (fg, bg) = if is_match {
                        (fg_match, bg_match)
                    } else {
                        (fg_text, bg_content)
                    };

                    // Accumulate consecutive chars with same color
                    let span_start = col;
                    let mut span = String::new();
                    span.push(ch);
                    col += 1;

                    while col < text_width {
                        let next_file_col = self.initial_column as usize + col;
                        let next_ch = line.get(next_file_col).copied().unwrap_or(' ');

                        let next_match = if search_len > 0 {
                            self.is_search_match(line, next_file_col, &search_chars)
                        } else {
                            false
                        };

                        let (next_fg, next_bg) = if next_match {
                            (fg_match, bg_match)
                        } else {
                            (fg_text, bg_content)
                        };

                        if next_fg != fg || next_bg != bg {
                            break;
                        }

                        span.push(next_ch);
                        col += 1;
                    }

                    Self::write_span(
                        &mut writer,
                        text_start_col + span_start as u16,
                        screen_row,
                        fg,
                        bg,
                        &span,
                    );
                }
            } else {
                // Empty row past end of file
                let blank: String = " ".repeat(text_width);
                Self::write_span(
                    &mut writer,
                    text_start_col,
                    screen_row,
                    fg_line_nr,
                    bg_content,
                    &blank,
                );
            }
        }

        // Fill remaining content rows
        let rendered_content_rows = file_matrix_row_end.saturating_sub(file_matrix_row_start);
        if rendered_content_rows < content_rows {
            let blank_line_nr: String = " ".repeat(row_lines_length);
            let blank_content: String =
                " ".repeat(content_w.saturating_sub(row_lines_length as u16) as usize);
            for i in rendered_content_rows..content_rows {
                let screen_row = content_start_row + i;
                Self::write_span(
                    &mut writer,
                    content_start,
                    screen_row,
                    fg_line_nr,
                    bg_line_nr,
                    &blank_line_nr,
                );
                Self::write_span(
                    &mut writer,
                    content_start + row_lines_length as u16,
                    screen_row,
                    fg_line_nr,
                    bg_content,
                    &blank_content,
                );
            }
        }

        // --- Status bar ---
        self.render_status_bar(&mut writer, content_start, content_w);

        queue!(writer, style::ResetColor).unwrap();
        if self.show_cursor {
            queue!(writer, cursor::Show, cursor::MoveTo(last_col, last_row)).unwrap();
        }
        writer.flush().unwrap();
    }

    fn is_search_match(&self, line: &[char], col: usize, search_chars: &[char]) -> bool {
        let search_len = search_chars.len();
        if search_len == 0 || col >= line.len() {
            return false;
        }
        let start = col.saturating_sub(search_len - 1);
        for match_start in start..=col {
            if match_start + search_len <= line.len() {
                let matches = (0..search_len).all(|k| {
                    line[match_start + k]
                        .to_lowercase()
                        .eq(search_chars[k].to_lowercase())
                });
                if matches && col >= match_start && col < match_start + search_len {
                    return true;
                }
            }
        }
        false
    }

    fn render_tab_bar(&self, writer: &mut BufWriter<io::Stdout>, start_col: u16, width: u16) {
        let bg_inactive = Color::Rgb {
            r: 20,
            g: 22,
            b: 20,
        };
        let bg_active = Color::Rgb {
            r: 40,
            g: 60,
            b: 40,
        };
        let fg_inactive = Color::Rgb {
            r: 120,
            g: 120,
            b: 120,
        };
        let fg_active = Color::Rgb {
            r: 220,
            g: 255,
            b: 220,
        };

        // Build tab content and track active ranges
        let mut tab_str = String::new();
        let mut active_ranges: Vec<(usize, usize)> = vec![];
        let mut pos = 0;

        for (name, is_active, is_modified) in &self.tab_names {
            let mod_indicator = if *is_modified { "● " } else { "" };
            let tab_text = format!(" {}{} ", mod_indicator, name);
            let tab_len = tab_text.chars().count();
            if *is_active {
                active_ranges.push((pos, pos + tab_len));
            }
            tab_str.push_str(&tab_text);
            tab_str.push('│');
            pos += tab_len + 1; // +1 for separator
        }

        // Pad to fill width
        let tab_chars: Vec<char> = tab_str.chars().collect();
        let total_len = width as usize;

        // Build spans: consecutive chars with same color
        let mut col = 0;
        while col < total_len {
            let is_active = active_ranges.iter().any(|(s, e)| col >= *s && col < *e);
            let (fg, bg) = if is_active {
                (fg_active, bg_active)
            } else {
                (fg_inactive, bg_inactive)
            };

            let span_start = col;
            let mut span = String::new();
            while col < total_len {
                let next_active = active_ranges.iter().any(|(s, e)| col >= *s && col < *e);
                if next_active != is_active {
                    break;
                }
                let ch = tab_chars.get(col).copied().unwrap_or(' ');
                span.push(ch);
                col += 1;
            }

            Self::write_span(writer, start_col + span_start as u16, 0, fg, bg, &span);
        }
    }

    fn render_status_bar(&self, writer: &mut BufWriter<io::Stdout>, start_col: u16, width: u16) {
        let status_row = self.rows - 1;

        let modified_indicator = if self.modified { "[+] " } else { "" };
        let left_part = format!(" {}{}", modified_indicator, self.filename);
        let info_part = format!(
            "Ln {}, Col {} | {} linhas",
            self.cursor_line, self.cursor_column, self.file_size
        );
        let mode_text = format!(" -- {} -- ", self.mode);
        let right_part = format!("{}  {}", info_part, mode_text);

        let padding = (width as usize).saturating_sub(left_part.len() + right_part.len());
        let status_line = format!("{}{}{}", left_part, " ".repeat(padding), right_part);

        // Truncate or pad to exact width
        let status_chars: Vec<char> = status_line.chars().collect();
        let mut final_str = String::with_capacity(width as usize);
        for i in 0..width as usize {
            final_str.push(status_chars.get(i).copied().unwrap_or(' '));
        }

        let bg_color = if self.mode == "INSERT" {
            Color::Rgb {
                r: 30,
                g: 50,
                b: 30,
            }
        } else {
            Color::Rgb {
                r: 20,
                g: 24,
                b: 20,
            }
        };
        let fg_color = Color::Rgb {
            r: 200,
            g: 200,
            b: 200,
        };

        Self::write_span(
            writer, start_col, status_row, fg_color, bg_color, &final_str,
        );
    }

    fn render_sidebar(&self, writer: &mut BufWriter<io::Stdout>, sidebar: &mut Sidebar) {
        let bg_sidebar = Color::Rgb {
            r: 18,
            g: 20,
            b: 18,
        };
        let fg_dir = Color::Rgb {
            r: 100,
            g: 180,
            b: 220,
        };
        let fg_file = Color::Rgb {
            r: 180,
            g: 180,
            b: 180,
        };
        let bg_selected = Color::Rgb {
            r: 40,
            g: 55,
            b: 40,
        };
        let fg_search = Color::Rgb {
            r: 200,
            g: 200,
            b: 200,
        };
        let bg_search = Color::Rgb {
            r: 25,
            g: 30,
            b: 25,
        };
        let fg_header = Color::Rgb {
            r: 100,
            g: 200,
            b: 130,
        };
        let bg_header = Color::Rgb {
            r: 25,
            g: 30,
            b: 25,
        };

        let sw = sidebar.width as usize;

        // Header row
        let header_text = format!(
            " {}",
            sidebar
                .root_path
                .file_name()
                .map(|n| n.to_string_lossy().to_string())
                .unwrap_or_else(|| sidebar.root_path.to_string_lossy().to_string())
        );
        let mut header_padded = String::with_capacity(sw);
        for (i, ch) in header_text.chars().enumerate() {
            if i >= sw {
                break;
            }
            header_padded.push(ch);
        }
        while header_padded.len() < sw {
            header_padded.push(' ');
        }
        Self::write_span(writer, 0, 0, fg_header, bg_header, &header_padded);

        // Search bar at row 1 if active
        let content_start_row: u16 = if sidebar.search_active { 2 } else { 1 };

        if sidebar.search_active {
            let search_display = format!(" / {}", sidebar.search_query);
            let mut search_padded = String::with_capacity(sw);
            for (i, ch) in search_display.chars().enumerate() {
                if i >= sw {
                    break;
                }
                search_padded.push(ch);
            }
            while search_padded.len() < sw {
                search_padded.push(' ');
            }
            Self::write_span(writer, 0, 1, fg_search, bg_search, &search_padded);
        }

        // File entries
        let entries = sidebar.flat_entries().to_vec();
        let available_rows = (self.rows - content_start_row) as usize;

        let scroll_offset = if sidebar.selected_index >= available_rows {
            sidebar.selected_index - available_rows + 1
        } else {
            0
        };

        for row in 0..available_rows {
            let screen_row = content_start_row + row as u16;
            let entry_idx = scroll_offset + row;

            if entry_idx < entries.len() {
                let entry = &entries[entry_idx];
                let is_selected = entry_idx == sidebar.selected_index;

                let indent = "  ".repeat(entry.depth);
                let icon = if entry.is_dir {
                    if entry.expanded {
                        "▼ "
                    } else {
                        "▶ "
                    }
                } else {
                    "  "
                };
                let line_text = format!(" {}{}{}", indent, icon, entry.name);

                // Pad or truncate to sidebar width
                let mut padded = String::with_capacity(sw);
                for (i, ch) in line_text.chars().enumerate() {
                    if i >= sw {
                        break;
                    }
                    padded.push(ch);
                }
                while padded.len() < sw {
                    padded.push(' ');
                }

                let bg = if is_selected { bg_selected } else { bg_sidebar };
                let fg = if entry.is_dir { fg_dir } else { fg_file };

                Self::write_span(writer, 0, screen_row, fg, bg, &padded);
            } else {
                let blank = " ".repeat(sw);
                Self::write_span(writer, 0, screen_row, fg_file, bg_sidebar, &blank);
            }
        }
    }

    // --- Public API ---

    pub fn offset_lines_number(&self) -> usize {
        let lines_length = self.file_matrix.len();
        lines_length.to_string().chars().count() + 2
    }

    fn offset_lines(&self, row_start: &u16, row_end: &u16) -> Vec<Vec<char>> {
        let row_lines_length = self.offset_lines_number();
        let rows_values = *row_start..*row_end;
        let mut rows: Vec<Vec<char>> = vec![];

        for row in rows_values {
            rows.push(
                format!(" {: >length$} ", row + 1, length = row_lines_length - 2)
                    .chars()
                    .collect(),
            );
        }

        rows
    }

    pub fn next_row(&mut self) {
        let content_rows = self.rows.saturating_sub(2);

        if self.initial_row >= (self.file_matrix.len() as u16).saturating_sub(content_rows) {
            return;
        }

        self.initial_row += 1;
    }

    pub fn previous_row(&mut self) {
        if self.initial_row > 0 {
            self.initial_row -= 1;
        }
    }

    pub fn next_column(&mut self) {
        let content_w = self.content_width();
        let (column_position, _) = cursor::position().unwrap();

        if column_position >= self.sidebar_width + content_w - 1 {
            self.initial_column += 1;
        }
    }

    pub fn previous_column(&mut self) {
        let (column_position, _) = cursor::position().unwrap();
        let min_col = self.sidebar_width + self.offset_lines_number() as u16;

        if column_position <= min_col && self.initial_column > 0 {
            self.initial_column -= 1;
        }
    }

    pub fn set_columns(&mut self, columns: u16) {
        self.columns = columns;
    }

    pub fn set_rows(&mut self, rows: u16) {
        self.rows = rows;
    }

    pub fn set_file_matrix(&mut self, file_matrix: Vec<Vec<char>>) {
        self.file_matrix = file_matrix;
    }

    pub fn set_mode(&mut self, mode: &str) {
        self.mode = String::from(mode);
    }

    pub fn set_modified(&mut self, modified: bool) {
        self.modified = modified;
    }

    pub fn set_cursor_info(&mut self, line: u16, column: u16) {
        self.cursor_line = line;
        self.cursor_column = column;
    }

    pub fn update_file_size(&mut self) {
        self.file_size = self.file_matrix.len();
    }

    pub fn reset_column(&mut self) {
        self.initial_column = 0;
    }

    pub fn reset_row(&mut self) {
        self.initial_row = 0;
    }

    pub fn get_absolute_row(&self, screen_row: u16) -> u16 {
        let content_row = screen_row.saturating_sub(1);
        self.initial_row + content_row
    }

    pub fn set_initial_column(&mut self, column: u16) {
        self.initial_column = column;
    }

    pub fn set_initial_row(&mut self, row: u16) {
        self.initial_row = row;
    }

    pub fn get_cursor_position(&self) -> u16 {
        let column_position = cursor::position().unwrap().0;
        let row_lines_length = self.offset_lines_number() as u16;

        self.initial_column + column_position.saturating_sub(self.sidebar_width + row_lines_length)
    }

    pub fn content_top_row(&self) -> u16 {
        1
    }
}
