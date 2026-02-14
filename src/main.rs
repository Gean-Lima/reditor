mod display;
mod buffer_file;
mod editor;

use std::io;

fn main() -> io::Result<()> {
    let mut buffer_file = buffer_file::BufferFile::new("example.json");
    let mut display = display::Display::new(buffer_file.filename.clone(), buffer_file.file_matrix.clone());

    let mut editor = editor::Editor::new(&mut buffer_file, &mut display);

    editor.run()?;

    Ok(())
}