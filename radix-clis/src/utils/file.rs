use crate::prelude::*;

pub fn write_ensuring_folder_exists(
    file_path: impl AsRef<Path>,
    contents: impl AsRef<[u8]>,
) -> Result<(), std::io::Error> {
    let file_path = file_path.as_ref();
    if let Some(parent_folder) = file_path.parent() {
        fs::create_dir_all(parent_folder)?;
    }
    fs::write(&file_path, contents)?;
    Ok(())
}
