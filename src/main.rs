mod buffer_file;
mod display;
mod editor;
mod sidebar;
mod syntax;
mod welcome;
mod workspace;

use std::env;
use std::io;
use std::path::PathBuf;

fn main() -> io::Result<()> {
    let args: Vec<String> = env::args().collect();

    let mut workspace = workspace::Workspace::new();
    let mut sidebar_instance: Option<sidebar::Sidebar> = None;

    if args.len() > 1 {
        let path_arg = &args[1];
        let path = std::fs::canonicalize(PathBuf::from(path_arg))
            .unwrap_or_else(|_| PathBuf::from(path_arg));

        if path.is_dir() {
            // Open sidebar with directory
            sidebar_instance = Some(sidebar::Sidebar::new(path));
        } else if path.is_file() {
            // Open file directly
            workspace.open_file(&path.to_string_lossy());
            // Use parent dir for sidebar
            if let Some(parent) = path.parent() {
                sidebar_instance = Some(sidebar::Sidebar::new(parent.to_path_buf()));
            }
        } else {
            eprintln!("reditor: '{}' não encontrado", path_arg);
            return Ok(());
        }
    }
    // No args = welcome screen (no sidebar, no files)

    let mut editor = editor::Editor::new(workspace, sidebar_instance);
    editor.run()?;

    Ok(())
}
