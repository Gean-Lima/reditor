use std::fs;
use std::fs::File;
use std::io::Read;

#[allow(dead_code)]
#[derive(Debug, Clone)]
pub struct BufferFile {
    pub filename: String,
    pub file_matrix: Vec<Vec<char>>,
    content: String,
}

impl BufferFile {
    pub fn new(path: &str) -> BufferFile {
        let file = File::open(path);
        let mut contents = String::new();

        file.unwrap().read_to_string(&mut contents).unwrap();

        BufferFile {
            filename: path.to_string(),
            file_matrix: BufferFile::get_file_matrix(&contents),
            content: contents,
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
            false
        } else if absolute_row > 0 {
            // Merge current line into previous line
            let current_line = self.file_matrix.remove(absolute_row);
            let previous_row = self.file_matrix.get_mut(absolute_row - 1).unwrap();
            previous_row.extend(current_line);
            true // indicates a line merge happened
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
    }

    pub fn save(&self) -> std::io::Result<()> {
        let content: String = self
            .file_matrix
            .iter()
            .map(|row| row.iter().collect::<String>())
            .collect::<Vec<String>>()
            .join("\n");

        fs::write(&self.filename, content)?;
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
}
