use crate::display::Display;
use crate::sidebar::Sidebar;
use crate::workspace::Workspace;
use crossterm::event::{Event, KeyCode, KeyEvent, KeyModifiers};
use crossterm::{cursor, event, execute, style, terminal};
use std::io;
use std::io::Write;

#[derive(PartialEq)]
enum EditorMode {
    Normal,
    Insert,
}

#[derive(PartialEq)]
enum Focus {
    Editor,
    Sidebar,
}

pub struct Editor {
    workspace: Workspace,
    display: Display,
    sidebar: Option<Sidebar>,
    mode: EditorMode,
    focus: Focus,
    show_welcome: bool,
    search_mode: bool,
    search_query: String,
    // Search state for save/restore
    search_saved_row: u16,
    search_saved_col: u16,
    search_saved_initial_row: u16,
    search_saved_initial_col: u16,
}

impl Editor {
    pub fn new(workspace: Workspace, sidebar: Option<Sidebar>) -> Editor {
        let show_welcome = !workspace.has_files();
        let display = Display::new();
        let initial_focus =
            if sidebar.as_ref().map(|s| s.visible).unwrap_or(false) && !workspace.has_files() {
                Focus::Sidebar
            } else {
                Focus::Editor
            };
        Editor {
            workspace,
            display,
            sidebar,
            mode: EditorMode::Normal,
            focus: initial_focus,
            show_welcome,
            search_mode: false,
            search_query: String::new(),
            search_saved_row: 0,
            search_saved_col: 0,
            search_saved_initial_row: 0,
            search_saved_initial_col: 0,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;
        terminal::enable_raw_mode()?;
        style::force_color_output(true);

        self.sync_display();
        self.render();

        self.position_cursor_at_start();

        loop {
            // Wait for first event
            let ev = event::read()?;

            // Process this event plus any pending ones before rendering
            let mut events = vec![ev];

            // Drain queued events (batching rapid key repeats)
            while event::poll(std::time::Duration::ZERO)? {
                events.push(event::read()?);
            }

            let mut should_break = false;

            for ev in events {
                let (column_size, row_size) = terminal::size()?;
                let (column_position, row_position) = cursor::position()?;

                match ev {
                    Event::Key(key) => {
                        if self.search_mode {
                            if self.handle_search_input(key)? {
                                continue;
                            }
                        }

                        // Global shortcuts
                        if key.modifiers.contains(KeyModifiers::CONTROL) {
                            match key.code {
                                KeyCode::Char('q') => {
                                    if self.handle_quit()? {
                                        should_break = true;
                                        break;
                                    }
                                    continue;
                                }
                                KeyCode::Char('s') => {
                                    self.workspace.save_active()?;
                                    self.sync_display();
                                    self.render();
                                    continue;
                                }
                                KeyCode::Char('t') => {
                                    self.toggle_sidebar();
                                    self.sync_display();
                                    self.render();
                                    self.position_cursor_at_start();
                                    continue;
                                }
                                KeyCode::Char('o') => {
                                    self.handle_open_file()?;
                                    continue;
                                }
                                KeyCode::Char('w') => {
                                    self.handle_close_tab()?;
                                    continue;
                                }
                                KeyCode::Char('f') => {
                                    if self.workspace.has_files() {
                                        self.search_mode = true;
                                        self.search_query.clear();
                                        // Save current position
                                        let (sc, sr) = cursor::position()?;
                                        self.search_saved_col = sc;
                                        self.search_saved_row = sr;
                                        self.search_saved_initial_row = self.display.initial_row;
                                        self.search_saved_initial_col = self.display.initial_column;
                                    }
                                    continue;
                                }
                                KeyCode::Tab | KeyCode::BackTab => {
                                    self.handle_tab_switch(key)?;
                                    continue;
                                }
                                _ => {}
                            }
                        }

                        if self.show_welcome && self.focus != Focus::Sidebar {
                            continue;
                        }

                        // Focus-specific handling
                        match self.focus {
                            Focus::Sidebar => {
                                self.handle_sidebar_input(key)?;
                            }
                            Focus::Editor => {
                                if !self.workspace.has_files() {
                                    continue;
                                }
                                match self.mode {
                                    EditorMode::Normal => {
                                        self.handle_normal_mode(
                                            key.code,
                                            column_position,
                                            row_position,
                                            row_size,
                                        )?;
                                    }
                                    EditorMode::Insert => {
                                        self.handle_insert_mode(
                                            key.code,
                                            column_position,
                                            row_position,
                                            column_size,
                                            row_size,
                                        )?;
                                    }
                                }
                            }
                        }
                    }
                    Event::Resize(w, h) => {
                        self.display.set_columns(w);
                        self.display.set_rows(h);
                    }
                    _ => {}
                }
            }

            if should_break {
                break;
            }

            self.update_status();
            self.render();

            // Draw search bar on top of status bar when in search mode
            if self.search_mode {
                self.render_search_bar().ok();
            }
        }

        terminal::disable_raw_mode()?;
        execute!(
            io::stdout(),
            cursor::Show,
            terminal::Clear(terminal::ClearType::All),
            terminal::LeaveAlternateScreen
        )?;

        Ok(())
    }

    fn sync_display(&mut self) {
        let sidebar_w = self
            .sidebar
            .as_ref()
            .map(|s| s.sidebar_offset())
            .unwrap_or(0);
        self.display.set_sidebar_width(sidebar_w);
        self.display.set_welcome(self.show_welcome);

        if let Some(buf) = self.workspace.active() {
            self.display.set_file_matrix(buf.file_matrix.clone());
            self.display.set_filename(buf.filename.clone());
            self.display.set_modified(buf.modified);
            self.display.set_initial_row(buf.initial_row);
            self.display.initial_column = buf.initial_column;
        }

        self.display.set_tab_names(self.workspace.tab_names());
        self.display.set_mode(if self.mode == EditorMode::Insert {
            "INSERT"
        } else {
            "NORMAL"
        });
        self.display
            .set_show_cursor(self.focus == Focus::Editor && self.workspace.has_files());
    }

    fn render(&mut self) {
        let search_q = if !self.search_query.is_empty() {
            Some(self.search_query.as_str())
        } else {
            None
        };
        self.display.show_display(self.sidebar.as_mut(), search_q);
    }

    fn update_status(&mut self) {
        if !self.workspace.has_files() {
            return;
        }

        let (_col_pos, row_pos) = cursor::position().unwrap_or((0, 0));
        let absolute_row = self.display.get_absolute_row(row_pos);
        let cursor_col = self.display.get_cursor_position();

        if let Some(buf) = self.workspace.active() {
            self.display.set_modified(buf.modified);
        }
        self.display
            .set_cursor_info(absolute_row + 1, cursor_col + 1);
        self.display.update_file_size();
    }

    fn position_cursor_at_start(&self) {
        let sidebar_w = self
            .sidebar
            .as_ref()
            .map(|s| s.sidebar_offset())
            .unwrap_or(0);
        let offset = self.display.offset_lines_number() as u16;
        let col = sidebar_w + offset;
        let row = self.display.content_top_row();
        execute!(io::stdout(), cursor::MoveTo(col, row)).unwrap();
    }

    fn toggle_sidebar(&mut self) {
        if let Some(sidebar) = &mut self.sidebar {
            if sidebar.visible && self.focus == Focus::Editor {
                // Sidebar already open — just switch focus to it
                self.focus = Focus::Sidebar;
            } else if sidebar.visible && self.focus == Focus::Sidebar {
                // Close sidebar
                sidebar.toggle_visible();
                self.focus = Focus::Editor;
            } else {
                // Open sidebar
                sidebar.toggle_visible();
                self.focus = Focus::Sidebar;
            }
        }
    }

    // --- Quit ---
    fn handle_quit(&mut self) -> io::Result<bool> {
        if self.workspace.is_any_modified() {
            match self.confirm_quit()? {
                QuitAction::Save => {
                    // Save all modified
                    for buf in &mut self.workspace.buffers {
                        if buf.modified {
                            buf.save()?;
                        }
                    }
                    return Ok(true);
                }
                QuitAction::Discard => return Ok(true),
                QuitAction::Cancel => {
                    self.sync_display();
                    self.render();
                    return Ok(false);
                }
            }
        }
        Ok(true)
    }

    fn confirm_quit(&self) -> io::Result<QuitAction> {
        let (_columns, rows) = terminal::size()?;
        let prompt = " Arquivos modificados! (s)alvar, (n)ão salvar, (c)ancelar: ";

        execute!(
            io::stdout(),
            cursor::MoveTo(0, rows - 1),
            style::SetBackgroundColor(style::Color::Rgb {
                r: 80,
                g: 30,
                b: 30,
            }),
            style::SetForegroundColor(style::Color::Rgb {
                r: 255,
                g: 220,
                b: 220,
            }),
        )?;

        for _ in 0.._columns {
            write!(io::stdout(), " ")?;
        }

        execute!(io::stdout(), cursor::MoveTo(0, rows - 1))?;
        write!(io::stdout(), "{}", prompt)?;
        io::stdout().flush()?;
        execute!(io::stdout(), style::ResetColor)?;

        loop {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Char('s') | KeyCode::Char('S') => return Ok(QuitAction::Save),
                    KeyCode::Char('n') | KeyCode::Char('N') => return Ok(QuitAction::Discard),
                    KeyCode::Char('c') | KeyCode::Char('C') | KeyCode::Esc => {
                        return Ok(QuitAction::Cancel)
                    }
                    _ => {}
                }
            }
        }
    }

    // --- Open file prompt ---
    fn handle_open_file(&mut self) -> io::Result<()> {
        let (_columns, rows) = terminal::size()?;
        let prompt = " Abrir arquivo: ";

        execute!(
            io::stdout(),
            cursor::MoveTo(0, rows - 1),
            style::SetBackgroundColor(style::Color::Rgb {
                r: 25,
                g: 35,
                b: 50,
            }),
            style::SetForegroundColor(style::Color::Rgb {
                r: 200,
                g: 220,
                b: 255,
            }),
        )?;

        for _ in 0.._columns {
            write!(io::stdout(), " ")?;
        }

        execute!(io::stdout(), cursor::MoveTo(0, rows - 1))?;
        write!(io::stdout(), "{}", prompt)?;
        io::stdout().flush()?;

        let mut input = String::new();
        loop {
            if let Event::Key(key) = event::read()? {
                match key.code {
                    KeyCode::Enter => {
                        execute!(io::stdout(), style::ResetColor)?;
                        let path = input.trim().to_string();
                        if !path.is_empty() && std::path::Path::new(&path).exists() {
                            self.workspace.open_file(&path);
                            self.show_welcome = false;
                            self.mode = EditorMode::Normal;
                            self.focus = Focus::Editor;
                            self.sync_display();
                            self.render();
                            self.position_cursor_at_start();
                        } else {
                            self.sync_display();
                            self.render();
                        }
                        return Ok(());
                    }
                    KeyCode::Esc => {
                        execute!(io::stdout(), style::ResetColor)?;
                        self.sync_display();
                        self.render();
                        return Ok(());
                    }
                    KeyCode::Char(c) => {
                        input.push(c);
                        write!(io::stdout(), "{}", c)?;
                        io::stdout().flush()?;
                    }
                    KeyCode::Backspace => {
                        if !input.is_empty() {
                            input.pop();
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                            write!(io::stdout(), " ")?;
                            execute!(io::stdout(), cursor::MoveLeft(1))?;
                            io::stdout().flush()?;
                        }
                    }
                    _ => {}
                }
            }
        }
    }

    // --- Close tab ---
    fn handle_close_tab(&mut self) -> io::Result<()> {
        if !self.workspace.has_files() {
            return Ok(());
        }

        // Check if active buffer is modified
        if let Some(buf) = self.workspace.active() {
            if buf.modified {
                match self.confirm_quit()? {
                    QuitAction::Save => {
                        self.workspace.save_active()?;
                    }
                    QuitAction::Discard => {}
                    QuitAction::Cancel => {
                        self.sync_display();
                        self.render();
                        return Ok(());
                    }
                }
            }
        }

        let was_empty = self.workspace.close_active();
        if was_empty || !self.workspace.has_files() {
            self.show_welcome = true;
        }

        self.display.reset_column();
        self.display.reset_row();
        self.sync_display();
        self.render();
        self.position_cursor_at_start();

        Ok(())
    }

    // --- Tab switching ---
    fn handle_tab_switch(&mut self, key: KeyEvent) -> io::Result<()> {
        if !self.workspace.has_files() {
            return Ok(());
        }

        // Save current cursor state
        self.save_cursor_state();

        if key.code == KeyCode::BackTab
            || (key.code == KeyCode::Tab && key.modifiers.contains(KeyModifiers::SHIFT))
        {
            self.workspace.prev_tab();
        } else {
            self.workspace.next_tab();
        }

        // Restore cursor state for new active buffer
        self.restore_cursor_state();
        self.sync_display();
        self.render();

        // Move cursor to saved position
        if let Some(buf) = self.workspace.active() {
            let sidebar_w = self
                .sidebar
                .as_ref()
                .map(|s| s.sidebar_offset())
                .unwrap_or(0);
            let offset = self.display.offset_lines_number() as u16;
            let col = sidebar_w + offset + buf.cursor_col;
            let row = self.display.content_top_row() + buf.cursor_row;
            execute!(io::stdout(), cursor::MoveTo(col, row))?;
        }

        Ok(())
    }

    fn save_cursor_state(&mut self) {
        let (_col_pos, row_pos) = cursor::position().unwrap_or((0, 0));
        let _abs_row = self.display.get_absolute_row(row_pos);
        let cursor_col = self.display.get_cursor_position();

        if let Some(buf) = self.workspace.active_mut() {
            buf.cursor_row = row_pos.saturating_sub(self.display.content_top_row());
            buf.cursor_col = cursor_col;
            buf.initial_row = self.display.initial_row;
            buf.initial_column = self.display.initial_column;
        }
    }

    fn restore_cursor_state(&mut self) {
        if let Some(buf) = self.workspace.active() {
            self.display.set_initial_row(buf.initial_row);
            self.display.initial_column = buf.initial_column;
        }
    }

    // --- Search ---
    fn handle_search_input(&mut self, key: KeyEvent) -> io::Result<bool> {
        match key.code {
            KeyCode::Esc => {
                // Restore original position
                self.search_mode = false;
                self.search_query.clear();
                self.display.set_initial_row(self.search_saved_initial_row);
                self.display
                    .set_initial_column(self.search_saved_initial_col);
                self.sync_display();
                self.render();
                execute!(
                    io::stdout(),
                    cursor::MoveTo(self.search_saved_col, self.search_saved_row)
                )?;
                return Ok(true);
            }
            KeyCode::Enter => {
                // Navigate to next match
                if !self.search_query.is_empty() {
                    self.navigate_to_next_match()?;
                }
                self.search_mode = false;
                // Keep search_query for highlighting
                return Ok(true);
            }
            KeyCode::Char(c) => {
                self.search_query.push(c);
                return Ok(true);
            }
            KeyCode::Backspace => {
                self.search_query.pop();
                return Ok(true);
            }
            _ => {}
        }
        Ok(true)
    }

    fn navigate_to_next_match(&mut self) -> io::Result<()> {
        let query: Vec<char> = self.search_query.to_lowercase().chars().collect();
        if query.is_empty() {
            return Ok(());
        }

        let buf = match self.workspace.active() {
            Some(b) => b,
            None => return Ok(()),
        };

        // Current position
        let (_cur_col_pos, cur_row_pos) = cursor::position()?;
        let current_row = self.display.get_absolute_row(cur_row_pos) as usize;
        let current_col = self.display.get_cursor_position() as usize;

        // Search from current position forward, wrap around
        let total_lines = buf.file_matrix.len();
        let search_col = current_col + 1; // start after current position

        for offset in 0..total_lines {
            let row_idx = (current_row + offset) % total_lines;
            let line = &buf.file_matrix[row_idx];
            let line_lower: Vec<char> = line.iter().flat_map(|c| c.to_lowercase()).collect();

            let start_col = if offset == 0 { search_col } else { 0 };

            // Search within this line
            let qlen = query.len();
            if line_lower.len() >= qlen {
                for col in start_col..=line_lower.len().saturating_sub(qlen) {
                    let matches = (0..qlen).all(|k| line_lower[col + k] == query[k]);
                    if matches {
                        // Found match at (row_idx, col)
                        self.jump_to_position(row_idx as u16, col as u16)?;
                        return Ok(());
                    }
                }
            }
        }

        Ok(())
    }

    fn jump_to_position(&mut self, file_row: u16, file_col: u16) -> io::Result<()> {
        let content_rows = self.display.rows.saturating_sub(2);
        let sidebar_w = self
            .sidebar
            .as_ref()
            .map(|s| s.sidebar_offset())
            .unwrap_or(0);
        let line_nr_w = self.display.offset_lines_number() as u16;
        let text_offset = sidebar_w + line_nr_w;
        let content_w = self.display.content_width().saturating_sub(line_nr_w);

        // Set initial_row so the target line is visible
        if file_row < self.display.initial_row
            || file_row >= self.display.initial_row + content_rows
        {
            // Center the target row
            let half = content_rows / 2;
            self.display.set_initial_row(file_row.saturating_sub(half));
        }

        // Set initial_column so the target column is visible
        if file_col < self.display.initial_column
            || file_col >= self.display.initial_column + content_w
        {
            self.display.set_initial_column(file_col.saturating_sub(5));
        }

        // Calculate screen position
        let screen_row = 1 + file_row.saturating_sub(self.display.initial_row);
        let screen_col = text_offset + file_col.saturating_sub(self.display.initial_column);

        self.sync_display();
        self.render();
        execute!(io::stdout(), cursor::MoveTo(screen_col, screen_row))?;

        Ok(())
    }

    fn render_search_bar(&self) -> io::Result<()> {
        let (columns, rows) = terminal::size()?;
        let sidebar_w = self
            .sidebar
            .as_ref()
            .map(|s| if s.visible { s.width } else { 0 })
            .unwrap_or(0);
        let start_col = sidebar_w;
        let width = columns.saturating_sub(sidebar_w) as usize;
        let prompt = format!(" Buscar: {}█", self.search_query);

        let bg = style::Color::Rgb {
            r: 25,
            g: 35,
            b: 50,
        };
        let fg = style::Color::Rgb {
            r: 200,
            g: 220,
            b: 255,
        };

        // Pad to width
        let prompt_chars: Vec<char> = prompt.chars().collect();
        let mut padded = String::with_capacity(width);
        for i in 0..width {
            padded.push(prompt_chars.get(i).copied().unwrap_or(' '));
        }

        execute!(
            io::stdout(),
            cursor::MoveTo(start_col, rows - 1),
            style::SetBackgroundColor(bg),
            style::SetForegroundColor(fg),
            style::Print(&padded),
            style::ResetColor,
        )?;

        Ok(())
    }

    // --- Sidebar input ---
    fn handle_sidebar_input(&mut self, key: KeyEvent) -> io::Result<()> {
        let sidebar = match &mut self.sidebar {
            Some(s) if s.visible => s,
            _ => {
                self.focus = Focus::Editor;
                return Ok(());
            }
        };

        if sidebar.search_active {
            match key.code {
                KeyCode::Esc => {
                    sidebar.clear_search();
                }
                KeyCode::Enter => {
                    sidebar.search_active = false;
                    // Keep search results visible
                }
                KeyCode::Char(c) => {
                    let mut q = sidebar.search_query.clone();
                    q.push(c);
                    sidebar.set_search_query(q);
                    return Ok(());
                }
                KeyCode::Backspace => {
                    let mut q = sidebar.search_query.clone();
                    q.pop();
                    sidebar.set_search_query(q);
                    return Ok(());
                }
                _ => {}
            }
            return Ok(());
        }

        match key.code {
            KeyCode::Up => sidebar.select_prev(),
            KeyCode::Down => sidebar.select_next(),
            KeyCode::Enter => {
                if sidebar.is_selected_dir() {
                    sidebar.toggle_selected_dir();
                } else if let Some(path) = sidebar.get_selected_path() {
                    let path_str = path.to_string_lossy().to_string();
                    self.workspace.open_file(&path_str);
                    self.show_welcome = false;
                    self.focus = Focus::Editor;
                    self.mode = EditorMode::Normal;
                    self.sync_display();
                    self.render();
                    self.position_cursor_at_start();
                    return Ok(());
                }
            }
            KeyCode::Right => {
                // Switch focus to editor
                self.focus = Focus::Editor;
                if self.workspace.has_files() {
                    self.position_cursor_at_start();
                }
                return Ok(());
            }
            KeyCode::Left => {
                // Collapse selected dir
                if sidebar.is_selected_dir() {
                    sidebar.toggle_selected_dir();
                }
            }
            KeyCode::Esc => {
                self.focus = Focus::Editor;
                if self.workspace.has_files() {
                    self.position_cursor_at_start();
                }
                return Ok(());
            }
            KeyCode::Char('/') => {
                sidebar.search_active = true;
                sidebar.search_query.clear();
            }
            _ => {}
        }

        Ok(())
    }

    // --- Navigation (shared) ---
    fn handle_navigation(
        &mut self,
        key_code: &KeyCode,
        column_position: u16,
        row_position: u16,
        row_size: u16,
    ) -> io::Result<bool> {
        let content_top = self.display.content_top_row();
        let content_bottom = row_size.saturating_sub(2); // status bar

        match key_code {
            KeyCode::Up => {
                if row_position > content_top {
                    execute!(io::stdout(), cursor::MoveUp(1))?;
                } else {
                    self.display.previous_row();
                }
                Ok(true)
            }
            KeyCode::Down => {
                if row_position < content_bottom {
                    execute!(io::stdout(), cursor::MoveDown(1))?;
                } else {
                    self.display.next_row();
                }
                Ok(true)
            }
            KeyCode::Right => {
                self.display.next_column(column_position);
                execute!(io::stdout(), cursor::MoveRight(1))?;
                Ok(true)
            }
            KeyCode::Left => {
                let sidebar_w = self
                    .sidebar
                    .as_ref()
                    .map(|s| s.sidebar_offset())
                    .unwrap_or(0);
                let min_col = sidebar_w + self.display.offset_lines_number() as u16;
                if column_position > min_col {
                    execute!(io::stdout(), cursor::MoveLeft(1))?;
                } else {
                    self.display.previous_column(column_position);
                }
                Ok(true)
            }
            KeyCode::Home => {
                let sidebar_w = self
                    .sidebar
                    .as_ref()
                    .map(|s| s.sidebar_offset())
                    .unwrap_or(0);
                let offset = sidebar_w + self.display.offset_lines_number() as u16;
                self.display.reset_column();
                execute!(io::stdout(), cursor::MoveTo(offset, row_position))?;
                Ok(true)
            }
            KeyCode::End => {
                let absolute_row = self.display.get_absolute_row(row_position);
                if let Some(buf) = self.workspace.active() {
                    let line_len = buf.get_line_length(absolute_row);
                    let sidebar_w = self
                        .sidebar
                        .as_ref()
                        .map(|s| s.sidebar_offset())
                        .unwrap_or(0);
                    let offset = sidebar_w + self.display.offset_lines_number() as u16;
                    let (col_size, _) = terminal::size()?;
                    let visible_area = col_size.saturating_sub(offset);

                    if line_len <= visible_area {
                        self.display.reset_column();
                        execute!(
                            io::stdout(),
                            cursor::MoveTo(offset + line_len, row_position)
                        )?;
                    } else {
                        self.display
                            .set_initial_column(line_len.saturating_sub(visible_area));
                        execute!(io::stdout(), cursor::MoveTo(col_size - 1, row_position))?;
                    }
                }
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    // --- Normal mode ---
    fn handle_normal_mode(
        &mut self,
        key_code: KeyCode,
        column_position: u16,
        row_position: u16,
        row_size: u16,
    ) -> io::Result<()> {
        if self.handle_navigation(&key_code, column_position, row_position, row_size)? {
            return Ok(());
        }

        match key_code {
            KeyCode::Char('i') => {
                self.mode = EditorMode::Insert;
                self.display.set_mode("INSERT");
            }
            _ => {}
        }

        Ok(())
    }

    // --- Insert mode ---
    fn handle_insert_mode(
        &mut self,
        key_code: KeyCode,
        column_position: u16,
        row_position: u16,
        _column_size: u16,
        row_size: u16,
    ) -> io::Result<()> {
        if key_code == KeyCode::Esc {
            self.mode = EditorMode::Normal;
            self.display.set_mode("NORMAL");
            return Ok(());
        }

        if self.handle_navigation(&key_code, column_position, row_position, row_size)? {
            return Ok(());
        }

        let absolute_row = self.display.get_absolute_row(row_position);
        let content_top = self.display.content_top_row();

        match key_code {
            KeyCode::Char(c) => {
                let cursor_col = self.display.get_cursor_position();
                if let Some(buf) = self.workspace.active_mut() {
                    buf.add_char(c, cursor_col, absolute_row);
                    self.display.set_file_matrix(buf.file_matrix.clone());
                }
                self.display.next_column(column_position);
                execute!(io::stdout(), cursor::MoveRight(1))?;
            }
            KeyCode::Backspace => {
                let cursor_col = self.display.get_cursor_position();
                let merged = if let Some(buf) = self.workspace.active_mut() {
                    let m = buf.remove_char(cursor_col, absolute_row);
                    self.display.set_file_matrix(buf.file_matrix.clone());
                    m
                } else {
                    false
                };

                if merged {
                    if row_position > content_top {
                        execute!(io::stdout(), cursor::MoveUp(1))?;
                    } else {
                        self.display.previous_row();
                    }
                } else if cursor_col > 0 {
                    self.display.previous_column(column_position);
                    let sidebar_w = self
                        .sidebar
                        .as_ref()
                        .map(|s| s.sidebar_offset())
                        .unwrap_or(0);
                    let min_col = sidebar_w + self.display.offset_lines_number() as u16;
                    if column_position > min_col {
                        execute!(io::stdout(), cursor::MoveLeft(1))?;
                    }
                }
            }
            KeyCode::Enter => {
                let cursor_col = self.display.get_cursor_position();
                if let Some(buf) = self.workspace.active_mut() {
                    buf.split_line(cursor_col, absolute_row);
                    self.display.set_file_matrix(buf.file_matrix.clone());
                }

                let sidebar_w = self
                    .sidebar
                    .as_ref()
                    .map(|s| s.sidebar_offset())
                    .unwrap_or(0);
                let offset = sidebar_w + self.display.offset_lines_number() as u16;
                let content_bottom = row_size.saturating_sub(2);

                self.display.reset_column();

                if row_position < content_bottom {
                    execute!(io::stdout(), cursor::MoveTo(offset, row_position + 1))?;
                } else {
                    self.display.next_row();
                    execute!(io::stdout(), cursor::MoveTo(offset, row_position))?;
                }
            }
            KeyCode::Tab => {
                let cursor_col = self.display.get_cursor_position();
                if let Some(buf) = self.workspace.active_mut() {
                    for i in 0..4 {
                        buf.add_char(' ', cursor_col + i, absolute_row);
                    }
                    self.display.set_file_matrix(buf.file_matrix.clone());
                }
                for j in 0..4 {
                    self.display.next_column(column_position + j);
                }
                execute!(io::stdout(), cursor::MoveRight(4))?;
            }
            _ => {}
        }

        Ok(())
    }
}

enum QuitAction {
    Save,
    Discard,
    Cancel,
}
