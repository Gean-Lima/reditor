use crossterm::style::Color;
use crossterm::{cursor, execute, style, terminal};
use std::io;

pub struct Display {
    filename: String,
    file_matrix: Vec<Vec<char>>,
    columns: u16,
    rows: u16,
    initial_row: u16,
    initial_column: u16,
    last_position_cursor_row: u16,
    last_position_cursor_column: u16,
    mode: String,
    modified: bool,
    cursor_line: u16,
    cursor_column: u16,
    file_size: usize,
}

pub struct DisplayChar {
    character: char,
    color: Color,
}

impl Display {
    pub fn new(filename: String, file_matrix: Vec<Vec<char>>) -> Display {
        let (columns, rows) = terminal::size().unwrap();

        let file_size = file_matrix.len();

        Display {
            filename,
            file_matrix,
            columns,
            rows,
            initial_row: 0,
            initial_column: 0,
            last_position_cursor_row: 0,
            last_position_cursor_column: 0,
            mode: String::from("NORMAL"),
            modified: false,
            cursor_line: 1,
            cursor_column: 1,
            file_size,
        }
    }

    pub fn show_display(&self) {
        let (last_column_position, last_row_position) = cursor::position().unwrap();

        execute!(io::stdout(), cursor::Hide).unwrap();

        let mut file_matrix_row_start = self.initial_row;
        let mut file_matrix_row_end = file_matrix_row_start + self.rows - 1;

        if file_matrix_row_end as usize >= self.file_matrix.len() {
            let leftover = file_matrix_row_end as usize - self.file_matrix.len();
            file_matrix_row_start = file_matrix_row_start - leftover as u16;
            file_matrix_row_end = file_matrix_row_end - leftover as u16;
        }

        let file_matrix = {
            let part_file = self.file_matrix
                [file_matrix_row_start as usize..file_matrix_row_end as usize]
                .to_vec();
            let mut matrix: Vec<Vec<char>> = vec![];

            for line in part_file {
                if line.get(self.initial_column as usize).is_some() {
                    matrix.push(line[self.initial_column as usize..].to_vec());
                    continue;
                }

                matrix.push(vec![]);
            }

            matrix
        };

        let row_lines_length = self.offset_lines_number();
        let row_lines = self.offset_lines(&file_matrix_row_start, &file_matrix_row_end);

        let display = {
            let mut matrix: Vec<Vec<DisplayChar>> = vec![];

            for index in 0..self.rows {
                let mut row: Vec<DisplayChar> = vec![];

                for index_col in 0..self.columns {
                    // ultima linha, exibe status bar completa
                    if index == self.rows - 1 {
                        let modified_indicator = if self.modified { "[+] " } else { "" };
                        let left_part = format!(" {}{}", modified_indicator, self.filename);
                        let info_part = format!(
                            "Ln {}, Col {} | {} linhas",
                            self.cursor_line, self.cursor_column, self.file_size
                        );
                        let mode_text = format!(" -- {} -- ", self.mode);
                        let right_part = format!("{}  {}", info_part, mode_text);

                        let padding = (self.columns as usize)
                            .saturating_sub(left_part.len() + right_part.len());
                        let status_line =
                            format!("{}{}{}", left_part, " ".repeat(padding), right_part);

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

                        if let Some(char) = status_line.chars().nth(index_col as usize) {
                            row.push(DisplayChar {
                                character: char,
                                color: bg_color,
                            });
                            continue;
                        }

                        row.push(DisplayChar {
                            character: ' ',
                            color: bg_color,
                        });

                        continue;
                    }

                    // exibe o numéro de cada linha
                    if index_col < row_lines_length as u16 {
                        row.push(DisplayChar {
                            character: row_lines[index as usize][index_col as usize],
                            color: Color::Rgb {
                                r: 10,
                                g: 12,
                                b: 10,
                            },
                        });

                        continue;
                    }

                    // exibe o conteúdo do arquivo
                    if let Some(chars) = file_matrix.get(index as usize) {
                        if let Some(char) = chars.get(index_col as usize - row_lines_length) {
                            row.push(DisplayChar {
                                character: char.clone(),
                                color: Color::Rgb {
                                    r: 15,
                                    g: 18,
                                    b: 15,
                                },
                            });

                            continue;
                        }
                    }

                    row.push(DisplayChar {
                        character: ' ',
                        color: Color::Rgb {
                            r: 15,
                            g: 18,
                            b: 15,
                        },
                    });
                }

                matrix.push(row);
            }

            matrix
        };

        for (index, row) in display.iter().enumerate() {
            for (index_c, col) in row.iter().enumerate() {
                execute!(
                    io::stdout(),
                    cursor::MoveTo(index_c as u16, index as u16),
                    style::SetBackgroundColor(col.color),
                    style::Print(col.character),
                    style::ResetColor
                )
                .unwrap();
            }
        }

        execute!(io::stdout(), cursor::Show).unwrap();

        execute!(
            io::stdout(),
            cursor::MoveTo(last_column_position, last_row_position)
        )
        .unwrap();
    }

    pub fn offset_lines_number(&self) -> usize {
        let lines_length = self.file_matrix.len();
        let row_lines_length = lines_length.to_string().chars().count() + 2;

        row_lines_length
    }

    fn offset_lines(&self, row_start: &u16, row_end: &u16) -> Vec<Vec<char>> {
        let row_lines_length = self.offset_lines_number();
        let row_lines = {
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
        };

        row_lines
    }

    pub fn next_row(&mut self) {
        let row_size = terminal::size().unwrap().1;

        if self.initial_row >= ((self.file_matrix.len() as u16) - row_size + 1) {
            return;
        }

        let (column_position, row_position) = cursor::position().unwrap();

        self.last_position_cursor_row = row_position;
        self.last_position_cursor_column = column_position;
        self.initial_row = self.initial_row + 1;
        self.show_display();
        execute!(
            io::stdout(),
            cursor::MoveTo(
                self.last_position_cursor_column,
                self.last_position_cursor_row
            )
        )
        .unwrap();
    }

    pub fn previous_row(&mut self) {
        let (column_position, row_position) = cursor::position().unwrap();

        if row_position == 0 && self.initial_row > 0 {
            self.last_position_cursor_row = row_position;
            self.last_position_cursor_column = column_position;
            self.initial_row = self.initial_row - 1;
            self.show_display();
            execute!(
                io::stdout(),
                cursor::MoveTo(
                    self.last_position_cursor_column,
                    self.last_position_cursor_row
                )
            )
            .unwrap();
        }
    }

    pub fn next_column(&mut self) {
        let column_size = terminal::size().unwrap().0;
        let (column_position, row_position) = cursor::position().unwrap();

        if column_position == column_size - 1 {
            self.last_position_cursor_row = row_position;
            self.last_position_cursor_column = column_position;
            self.initial_column = self.initial_column + 1;
            self.show_display();
            execute!(
                io::stdout(),
                cursor::MoveTo(
                    self.last_position_cursor_column,
                    self.last_position_cursor_row
                )
            )
            .unwrap();
        }
    }

    pub fn previous_column(&mut self) {
        let (column_position, row_position) = cursor::position().unwrap();

        if column_position - 1 < self.offset_lines_number() as u16 && self.initial_column > 0 {
            self.last_position_cursor_row = row_position;
            self.last_position_cursor_column = column_position;
            self.initial_column = self.initial_column - 1;
            self.show_display();
            execute!(
                io::stdout(),
                cursor::MoveTo(
                    self.last_position_cursor_column,
                    self.last_position_cursor_row
                )
            )
            .unwrap();
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

    pub fn get_absolute_row(&self, screen_row: u16) -> u16 {
        self.initial_row + screen_row
    }

    pub fn set_initial_column(&mut self, column: u16) {
        self.initial_column = column;
    }

    pub fn get_cursor_position(&self) -> u16 {
        let column_position = cursor::position().unwrap().0;
        let row_lines_length = self.offset_lines_number() as u16;

        self.initial_column + column_position.saturating_sub(row_lines_length)
    }
}
