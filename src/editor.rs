use crate::buffer_file::BufferFile;
use crate::display::Display;
use crossterm::event::{Event, KeyCode, KeyModifiers};
use crossterm::{cursor, event, execute, style, terminal};
use std::io;
use std::io::Write;

#[derive(PartialEq)]
enum EditorMode {
    Normal,
    Insert,
}

pub struct Editor<'a> {
    buffer_file: &'a mut BufferFile,
    display: &'a mut Display,
    mode: EditorMode,
    modified: bool,
}

impl<'a> Editor<'a> {
    pub fn new(buffer_file: &'a mut BufferFile, display: &'a mut Display) -> Editor<'a> {
        Editor {
            buffer_file,
            display,
            mode: EditorMode::Normal,
            modified: false,
        }
    }

    pub fn run(&mut self) -> io::Result<()> {
        execute!(io::stdout(), terminal::EnterAlternateScreen)?;

        terminal::enable_raw_mode()?;
        style::force_color_output(true);

        self.display.set_mode("NORMAL");
        self.update_status();
        self.display.show_display();

        execute!(
            io::stdout(),
            cursor::MoveTo(self.display.offset_lines_number() as u16, 0)
        )?;

        loop {
            let (column_size, row_size) = terminal::size()?;
            let (column_position, row_position) = cursor::position()?;

            match event::read()? {
                Event::Key(key) => {
                    // Ctrl+Q sai em qualquer modo (com confirmação se modificado)
                    if key.code == KeyCode::Char('q') && key.modifiers == KeyModifiers::CONTROL {
                        if self.modified {
                            match self.confirm_quit()? {
                                QuitAction::Save => {
                                    self.buffer_file.save()?;
                                    break;
                                }
                                QuitAction::Discard => break,
                                QuitAction::Cancel => {}
                            }
                        } else {
                            break;
                        }
                    }

                    // Ctrl+S salva em qualquer modo
                    if key.code == KeyCode::Char('s') && key.modifiers == KeyModifiers::CONTROL {
                        self.buffer_file.save()?;
                        self.modified = false;
                        self.display.set_mode(if self.mode == EditorMode::Insert {
                            "INSERT"
                        } else {
                            "NORMAL"
                        });
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
                Event::Resize(_w, _h) => {
                    self.display.set_columns(column_size);
                    self.display.set_rows(row_size);
                }
                _ => {}
            }

            // Atualizar status bar após cada evento
            self.update_status();
            self.display.show_display();
        }

        terminal::disable_raw_mode()?;

        execute!(io::stdout(), terminal::Clear(terminal::ClearType::All))?;
        execute!(io::stdout(), terminal::LeaveAlternateScreen)?;

        Ok(())
    }

    fn update_status(&mut self) {
        let (_col_pos, row_pos) = cursor::position().unwrap_or((0, 0));
        let absolute_row = self.display.get_absolute_row(row_pos);
        let cursor_col = self.display.get_cursor_position();

        self.display.set_modified(self.modified);
        self.display
            .set_cursor_info(absolute_row + 1, cursor_col + 1);
        self.display.update_file_size();
    }

    fn confirm_quit(&self) -> io::Result<QuitAction> {
        let (_columns, rows) = terminal::size()?;
        let prompt = " Arquivo modificado! (s)alvar, (n)ão salvar, (c)ancelar: ";

        // Limpar a última linha e desenhar o prompt
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

        // Limpar a linha toda
        for _ in 0.._columns {
            write!(io::stdout(), " ")?;
        }

        execute!(io::stdout(), cursor::MoveTo(0, rows - 1),)?;
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

    fn handle_navigation(
        &mut self,
        key_code: &KeyCode,
        column_position: u16,
        row_position: u16,
        row_size: u16,
    ) -> io::Result<bool> {
        match key_code {
            KeyCode::Up => {
                self.display.previous_row();
                if row_position > 0 {
                    execute!(io::stdout(), cursor::MoveUp(1))?;
                }
                Ok(true)
            }
            KeyCode::Down => {
                if row_position < row_size - 2 {
                    execute!(io::stdout(), cursor::MoveDown(1))?;
                } else {
                    self.display.next_row();
                }
                Ok(true)
            }
            KeyCode::Right => {
                self.display.next_column();
                execute!(io::stdout(), cursor::MoveRight(1))?;
                Ok(true)
            }
            KeyCode::Left => {
                if column_position > self.display.offset_lines_number() as u16 {
                    execute!(io::stdout(), cursor::MoveLeft(1))?;
                } else {
                    self.display.previous_column();
                }
                Ok(true)
            }
            KeyCode::Home => {
                // Mover cursor para o início da linha (após offset de line numbers)
                let offset = self.display.offset_lines_number() as u16;
                self.display.reset_column();
                execute!(io::stdout(), cursor::MoveTo(offset, row_position))?;
                self.display.show_display();
                Ok(true)
            }
            KeyCode::End => {
                // Mover cursor para o final da linha
                let absolute_row = self.display.get_absolute_row(row_position);
                let line_len = self.buffer_file.get_line_length(absolute_row);
                let offset = self.display.offset_lines_number() as u16;
                let (col_size, _) = terminal::size()?;
                let visible_area = col_size - offset;

                if line_len <= visible_area {
                    // Linha cabe na tela
                    self.display.reset_column();
                    execute!(
                        io::stdout(),
                        cursor::MoveTo(offset + line_len, row_position)
                    )?;
                } else {
                    // Linha maior que a tela — scroll horizontal
                    self.display
                        .set_initial_column(line_len.saturating_sub(visible_area));
                    execute!(io::stdout(), cursor::MoveTo(col_size - 1, row_position))?;
                }
                self.display.show_display();
                Ok(true)
            }
            _ => Ok(false),
        }
    }

    fn handle_normal_mode(
        &mut self,
        key_code: KeyCode,
        column_position: u16,
        row_position: u16,
        row_size: u16,
    ) -> io::Result<()> {
        // Tenta tratar navegação primeiro
        if self.handle_navigation(&key_code, column_position, row_position, row_size)? {
            return Ok(());
        }

        match key_code {
            // Entrar no modo Insert
            KeyCode::Char('i') => {
                self.mode = EditorMode::Insert;
                self.display.set_mode("INSERT");
                self.display.show_display();
            }
            _ => {}
        }

        Ok(())
    }

    fn handle_insert_mode(
        &mut self,
        key_code: KeyCode,
        column_position: u16,
        row_position: u16,
        _column_size: u16,
        row_size: u16,
    ) -> io::Result<()> {
        // Voltar para o modo Normal
        if key_code == KeyCode::Esc {
            self.mode = EditorMode::Normal;
            self.display.set_mode("NORMAL");
            self.display.show_display();
            return Ok(());
        }

        // Tenta tratar navegação primeiro
        if self.handle_navigation(&key_code, column_position, row_position, row_size)? {
            return Ok(());
        }

        match key_code {
            // Inserção de caractere
            KeyCode::Char(c) => {
                let cursor_col = self.display.get_cursor_position();
                self.buffer_file.add_char(c, cursor_col, row_position);

                self.display
                    .set_file_matrix(self.buffer_file.file_matrix.clone());
                self.display.next_column();
                execute!(io::stdout(), cursor::MoveRight(1))?;
                self.display.show_display();
                self.modified = true;
            }

            // Backspace
            KeyCode::Backspace => {
                let cursor_col = self.display.get_cursor_position();
                let merged = self.buffer_file.remove_char(cursor_col, row_position);

                self.display
                    .set_file_matrix(self.buffer_file.file_matrix.clone());

                if merged {
                    // Linha foi mesclada com a anterior — mover cursor para cima
                    self.display.show_display();
                    if row_position > 0 {
                        execute!(io::stdout(), cursor::MoveUp(1))?;
                    } else {
                        self.display.previous_row();
                    }
                } else if cursor_col > 0 {
                    self.display.previous_column();
                    if column_position > self.display.offset_lines_number() as u16 {
                        execute!(io::stdout(), cursor::MoveLeft(1))?;
                    }
                    self.display.show_display();
                }
                self.modified = true;
            }

            // Enter
            KeyCode::Enter => {
                let cursor_col = self.display.get_cursor_position();
                self.buffer_file.split_line(cursor_col, row_position);

                self.display
                    .set_file_matrix(self.buffer_file.file_matrix.clone());
                self.display.show_display();

                // Mover cursor para o início da próxima linha
                let offset = self.display.offset_lines_number() as u16;
                if row_position < row_size - 2 {
                    execute!(io::stdout(), cursor::MoveTo(offset, row_position + 1))?;
                } else {
                    self.display.next_row();
                    execute!(io::stdout(), cursor::MoveTo(offset, row_position))?;
                }
                self.modified = true;
            }

            // Tab (4 espaços)
            KeyCode::Tab => {
                let cursor_col = self.display.get_cursor_position();
                for i in 0..4 {
                    self.buffer_file.add_char(' ', cursor_col + i, row_position);
                }

                self.display
                    .set_file_matrix(self.buffer_file.file_matrix.clone());
                for _ in 0..4 {
                    self.display.next_column();
                }
                execute!(io::stdout(), cursor::MoveRight(4))?;
                self.display.show_display();
                self.modified = true;
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
