use crate::buffer_file::BufferFile;

pub struct Workspace {
    pub buffers: Vec<BufferFile>,
    pub active_index: usize,
}

impl Workspace {
    pub fn new() -> Workspace {
        Workspace {
            buffers: vec![],
            active_index: 0,
        }
    }

    pub fn open_file(&mut self, path: &str) -> usize {
        // Check if file is already open
        for (i, buf) in self.buffers.iter().enumerate() {
            if buf.filename == path {
                self.active_index = i;
                return i;
            }
        }

        let buffer = BufferFile::new(path);
        self.buffers.push(buffer);
        self.active_index = self.buffers.len() - 1;
        self.active_index
    }

    pub fn close_active(&mut self) -> bool {
        if self.buffers.is_empty() {
            return false;
        }

        self.buffers.remove(self.active_index);

        if self.buffers.is_empty() {
            self.active_index = 0;
            return true;
        }

        if self.active_index >= self.buffers.len() {
            self.active_index = self.buffers.len() - 1;
        }

        false
    }

    pub fn next_tab(&mut self) {
        if self.buffers.len() > 1 {
            self.save_cursor_position();
            self.active_index = (self.active_index + 1) % self.buffers.len();
        }
    }

    pub fn prev_tab(&mut self) {
        if self.buffers.len() > 1 {
            self.save_cursor_position();
            if self.active_index == 0 {
                self.active_index = self.buffers.len() - 1;
            } else {
                self.active_index -= 1;
            }
        }
    }

    #[allow(dead_code)]
    pub fn switch_to(&mut self, index: usize) {
        if index < self.buffers.len() {
            self.save_cursor_position();
            self.active_index = index;
        }
    }

    fn save_cursor_position(&mut self) {
        // Cursor position is saved externally by editor before switching
    }

    pub fn active(&self) -> Option<&BufferFile> {
        self.buffers.get(self.active_index)
    }

    pub fn active_mut(&mut self) -> Option<&mut BufferFile> {
        self.buffers.get_mut(self.active_index)
    }

    pub fn has_files(&self) -> bool {
        !self.buffers.is_empty()
    }

    pub fn is_any_modified(&self) -> bool {
        self.buffers.iter().any(|b| b.modified)
    }

    pub fn save_active(&mut self) -> std::io::Result<()> {
        if let Some(buf) = self.active_mut() {
            buf.save()?;
        }
        Ok(())
    }

    pub fn tab_names(&self) -> Vec<(String, bool, bool)> {
        self.buffers
            .iter()
            .enumerate()
            .map(|(i, b)| {
                let name = b.short_name();
                let is_active = i == self.active_index;
                let is_modified = b.modified;
                (name, is_active, is_modified)
            })
            .collect()
    }
}
