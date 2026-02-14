use std::fs;
use std::path::PathBuf;

#[derive(Clone)]
pub struct FileEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub children: Vec<FileEntry>,
    pub expanded: bool,
    pub depth: usize,
}

pub struct Sidebar {
    pub root_path: PathBuf,
    pub entries: Vec<FileEntry>,
    pub selected_index: usize,
    pub visible: bool,
    pub width: u16,
    pub search_query: String,
    pub search_active: bool,
    flat_cache: Vec<FlatEntry>,
    cache_dirty: bool,
}

#[derive(Clone)]
pub struct FlatEntry {
    pub name: String,
    pub path: PathBuf,
    pub is_dir: bool,
    pub depth: usize,
    pub expanded: bool,
}

impl Sidebar {
    pub fn new(root_path: PathBuf) -> Sidebar {
        let entries = Sidebar::build_tree(&root_path, 0);
        let mut sidebar = Sidebar {
            root_path,
            entries,
            selected_index: 0,
            visible: true,
            width: 30,
            search_query: String::new(),
            search_active: false,
            flat_cache: vec![],
            cache_dirty: true,
        };
        sidebar.rebuild_flat_cache();
        sidebar
    }

    fn build_tree(path: &PathBuf, depth: usize) -> Vec<FileEntry> {
        let mut entries: Vec<FileEntry> = vec![];

        if let Ok(read_dir) = fs::read_dir(path) {
            let mut items: Vec<_> = read_dir.filter_map(|e| e.ok()).collect();
            items.sort_by(|a, b| {
                let a_is_dir = a.file_type().map(|t| t.is_dir()).unwrap_or(false);
                let b_is_dir = b.file_type().map(|t| t.is_dir()).unwrap_or(false);
                b_is_dir.cmp(&a_is_dir).then(
                    a.file_name()
                        .to_string_lossy()
                        .to_lowercase()
                        .cmp(&b.file_name().to_string_lossy().to_lowercase()),
                )
            });

            for item in items {
                let name = item.file_name().to_string_lossy().to_string();

                // Skip hidden files and common non-essential dirs
                if name.starts_with('.') || name == "target" || name == "node_modules" {
                    continue;
                }

                let item_path = item.path();
                let is_dir = item.file_type().map(|t| t.is_dir()).unwrap_or(false);

                entries.push(FileEntry {
                    name,
                    path: item_path,
                    is_dir,
                    children: vec![], // Lazy-loaded
                    expanded: false,
                    depth,
                });
            }
        }

        entries
    }

    fn rebuild_flat_cache(&mut self) {
        self.flat_cache.clear();

        if self.search_query.is_empty() {
            self.flatten_entries(&self.entries.clone());
        } else {
            let query = self.search_query.to_lowercase();
            self.flatten_entries_filtered(&self.entries.clone(), &query);
        }

        self.cache_dirty = false;
    }

    fn flatten_entries(&mut self, entries: &[FileEntry]) {
        for entry in entries {
            self.flat_cache.push(FlatEntry {
                name: entry.name.clone(),
                path: entry.path.clone(),
                is_dir: entry.is_dir,
                depth: entry.depth,
                expanded: entry.expanded,
            });

            if entry.is_dir && entry.expanded {
                self.flatten_entries(&entry.children);
            }
        }
    }

    fn flatten_entries_filtered(&mut self, entries: &[FileEntry], query: &str) {
        for entry in entries {
            let matches = entry.name.to_lowercase().contains(query);

            if matches || entry.is_dir {
                if matches {
                    self.flat_cache.push(FlatEntry {
                        name: entry.name.clone(),
                        path: entry.path.clone(),
                        is_dir: entry.is_dir,
                        depth: entry.depth,
                        expanded: entry.expanded,
                    });
                }

                if entry.is_dir && entry.expanded {
                    self.flatten_entries_filtered(&entry.children, query);
                }
            }
        }
    }

    pub fn flat_entries(&mut self) -> &[FlatEntry] {
        if self.cache_dirty {
            self.rebuild_flat_cache();
        }
        &self.flat_cache
    }

    pub fn flat_len(&mut self) -> usize {
        if self.cache_dirty {
            self.rebuild_flat_cache();
        }
        self.flat_cache.len()
    }

    pub fn toggle_visible(&mut self) {
        self.visible = !self.visible;
    }

    pub fn select_next(&mut self) {
        let len = self.flat_len();
        if len > 0 && self.selected_index < len - 1 {
            self.selected_index += 1;
        }
    }

    pub fn select_prev(&mut self) {
        if self.selected_index > 0 {
            self.selected_index -= 1;
        }
    }

    pub fn get_selected_path(&mut self) -> Option<PathBuf> {
        let idx = self.selected_index;
        let entries = self.flat_entries();
        entries.get(idx).map(|e| e.path.clone())
    }

    pub fn is_selected_dir(&mut self) -> bool {
        let idx = self.selected_index;
        let entries = self.flat_entries();
        entries.get(idx).map(|e| e.is_dir).unwrap_or(false)
    }

    pub fn toggle_selected_dir(&mut self) {
        if self.cache_dirty {
            self.rebuild_flat_cache();
        }

        if let Some(flat) = self.flat_cache.get(self.selected_index) {
            if !flat.is_dir {
                return;
            }
            let target_path = flat.path.clone();
            let target_depth = flat.depth;

            // Find and toggle in the actual tree
            Self::toggle_dir_in_tree(&mut self.entries, &target_path, target_depth);
            self.cache_dirty = true;
            self.rebuild_flat_cache();
        }
    }

    fn toggle_dir_in_tree(entries: &mut Vec<FileEntry>, target: &PathBuf, _depth: usize) -> bool {
        for entry in entries.iter_mut() {
            if entry.path == *target && entry.is_dir {
                entry.expanded = !entry.expanded;
                if entry.expanded && entry.children.is_empty() {
                    entry.children = Sidebar::build_tree(&entry.path, entry.depth + 1);
                }
                return true;
            }
            if entry.is_dir && entry.expanded {
                if Self::toggle_dir_in_tree(&mut entry.children, target, _depth) {
                    return true;
                }
            }
        }
        false
    }

    pub fn set_search_query(&mut self, query: String) {
        self.search_query = query;
        self.cache_dirty = true;
        self.selected_index = 0;
    }

    pub fn clear_search(&mut self) {
        self.search_query.clear();
        self.search_active = false;
        self.cache_dirty = true;
        self.selected_index = 0;
    }

    pub fn sidebar_offset(&self) -> u16 {
        if self.visible {
            self.width
        } else {
            0
        }
    }
}
