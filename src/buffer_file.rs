use std::fs;
use std::fs::File;
use std::io::Read;

#[derive(Debug, Clone)]
pub struct BufferFile {
    pub filename: String,
    pub file_matrix: Vec<Vec<char>>,
    pub modified: bool,
    pub cursor_row: u16,
    pub cursor_col: u16,
    pub initial_row: u16,
    pub initial_column: u16,
}

impl BufferFile {
    pub fn new(path: &str) -> BufferFile {
        let file = File::open(path);
        let mut contents = String::new();

        file.unwrap().read_to_string(&mut contents).unwrap();

        BufferFile {
            filename: path.to_string(),
            file_matrix: BufferFile::get_file_matrix(&contents),
            modified: false,
            cursor_row: 0,
            cursor_col: 0,
            initial_row: 0,
            initial_column: 0,
        }
    }

    #[allow(dead_code)]
    pub fn new_empty(filename: &str) -> BufferFile {
        BufferFile {
            filename: filename.to_string(),
            file_matrix: vec![vec![]],
            modified: false,
            cursor_row: 0,
            cursor_col: 0,
            initial_row: 0,
            initial_column: 0,
        }
    }

    fn get_file_matrix(content: &String) -> Vec<Vec<char>> {
        let mut matrix: Vec<Vec<char>> = vec![];

        content.lines().for_each(|line| {
            let mut row: Vec<char> = vec![];

            line.chars().for_each(|character| {
                row.push(character);
            });

            matrix.push(row);
        });

        if matrix.is_empty() {
            matrix.push(vec![]);
        }

        matrix
    }

    pub fn add_char(&mut self, character: char, column: u16, row: u16) {
        let absolute_row = row as usize;

        if absolute_row >= self.file_matrix.len() {
            return;
        }

        let file_row = self.file_matrix.get_mut(absolute_row).unwrap();

        if (column as usize) < file_row.len() {
            file_row.insert(column as usize, character);
        } else {
            file_row.push(character);
        }
        self.modified = true;
    }

    pub fn remove_char(&mut self, column: u16, row: u16) -> bool {
        let absolute_row = row as usize;

        if absolute_row >= self.file_matrix.len() {
            return false;
        }

        let col = column as usize;

        if col > 0 {
            let file_row = self.file_matrix.get_mut(absolute_row).unwrap();
            if col <= file_row.len() {
                file_row.remove(col - 1);
            }
            self.modified = true;
            false
        } else if absolute_row > 0 {
            let current_line = self.file_matrix.remove(absolute_row);
            let previous_row = self.file_matrix.get_mut(absolute_row - 1).unwrap();
            previous_row.extend(current_line);
            self.modified = true;
            true
        } else {
            false
        }
    }

    pub fn split_line(&mut self, column: u16, row: u16) {
        let absolute_row = row as usize;

        if absolute_row >= self.file_matrix.len() {
            return;
        }

        let file_row = self.file_matrix.get_mut(absolute_row).unwrap();
        let col = column as usize;

        let new_line = if col < file_row.len() {
            file_row.split_off(col)
        } else {
            vec![]
        };

        self.file_matrix.insert(absolute_row + 1, new_line);
        self.modified = true;
    }

    pub fn save(&mut self) -> std::io::Result<()> {
        let content: String = self
            .file_matrix
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        fs::write(&self.filename, content)?;
        self.modified = false;
        Ok(())
    }

    pub fn get_line_length(&self, row: u16) -> u16 {
        let absolute_row = row as usize;
        if absolute_row < self.file_matrix.len() {
            self.file_matrix[absolute_row].len() as u16
        } else {
            0
        }
    }

    pub fn short_name(&self) -> String {
        std::path::Path::new(&self.filename)
            .file_name()
            .map(|n| n.to_string_lossy().to_string())
            .unwrap_or_else(|| self.filename.clone())
    }
}
