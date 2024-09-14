use crate::internal_prelude::*;
// This module is only included if std exists, so it's fine to import it
use std::{ffi::OsString, fs, path::*};

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AlignerExecutionMode {
    Write,
    Assert,
}

#[derive(Copy, Clone, PartialEq, Eq)]
pub enum AlignerFolderMode {
    /// In this case:
    /// * The folder is initially wiped
    /// * The folder is checked to have no extra content when the FolderContentAligner is dropped
    ExpectNoOtherContent,
    /// * In this case, the folder is not initially wiped
    /// * There is no check of completeness on drop
    ExpectOtherContent,
}

/// This is for use in tests, to perform two actions:
/// * Write that the output data to folders
/// * Verify that this content matches the output data
pub struct FolderContentAligner {
    folder: PathBuf,
    execution_mode: AlignerExecutionMode,
    folder_mode: AlignerFolderMode,
    files_touched: IndexSet<OsString>,
    folders_touched: IndexSet<OsString>,
}

impl FolderContentAligner {
    pub fn new(
        folder: PathBuf,
        execution_mode: AlignerExecutionMode,
        folder_mode: AlignerFolderMode,
    ) -> Self {
        // NOTE: In future, we could improve this behaviour to avoid creating churn if files don't need to change
        match execution_mode {
            AlignerExecutionMode::Write => {
                if folder_mode == AlignerFolderMode::ExpectNoOtherContent && folder.exists() {
                    std::fs::remove_dir_all(&folder).unwrap();
                }
                std::fs::create_dir_all(&folder).unwrap();
            }
            AlignerExecutionMode::Assert => {}
        }
        Self {
            folder,
            execution_mode,
            folder_mode,
            files_touched: indexset!(),
            folders_touched: indexset!(),
        }
    }

    pub fn put_file<F: AsRef<str>, C: AsRef<[u8]>>(&mut self, file: F, contents: C) {
        let file = file.as_ref();
        let path = self.folder.join(file);
        self.files_touched.insert(file.into());
        match self.execution_mode {
            AlignerExecutionMode::Write => fs::write(path, contents).unwrap(),
            AlignerExecutionMode::Assert => {
                let actual_contents = fs::read(&path).unwrap_or_else(|err| {
                    panic!(
                        "File {} could not be read: {:?}",
                        path.to_string_lossy(),
                        err
                    );
                });
                if &actual_contents != contents.as_ref() {
                    panic!(
                        "File {} did not match the expected contents",
                        path.to_string_lossy()
                    )
                }
            }
        }
    }

    pub fn register_child_folder<F: AsRef<str>>(
        &mut self,
        child_folder: F,
        folder_mode: AlignerFolderMode,
    ) -> FolderContentAligner {
        let child_folder = child_folder.as_ref();
        let path = self.folder.join(child_folder);
        let folder_aligner = FolderContentAligner::new(path, self.execution_mode, folder_mode);
        self.folders_touched.insert(child_folder.into());
        folder_aligner
    }

    /// This is run automatically on drop if the folder mode is `AlignerFolderMode::ExpectNoOtherContent`
    fn verify_no_extra_content_exists(&self) {
        match self.execution_mode {
            AlignerExecutionMode::Write => {}
            AlignerExecutionMode::Assert => {
                // If the folder doesn't exist, then it can't contain any contents, so this check is fine
                if !self.folder.exists() {
                    return;
                }
                for entry in walkdir::WalkDir::new(&self.folder)
                    .min_depth(1)
                    .max_depth(1)
                {
                    let entry = entry.unwrap();
                    let file_name = entry.file_name();
                    let is_file = entry.file_type().is_file();
                    let is_folder = entry.file_type().is_dir();
                    match (is_file, is_folder) {
                        (true, false) => {
                            if !self.files_touched.contains(file_name) {
                                panic!("File {} should not exist", entry.path().to_string_lossy())
                            }
                        }
                        (false, true) => {
                            if !self.folders_touched.contains(file_name) {
                                panic!("Folder {} should not exist", entry.path().to_string_lossy())
                            }
                        }
                        (true, true) => {
                            panic!(
                                "Path {} was unexpectedly both a file and a folder",
                                entry.path().to_string_lossy()
                            )
                        }
                        (false, false) => {
                            panic!(
                                "Path {} was unexpectedly neither a file nor a folder",
                                entry.path().to_string_lossy()
                            )
                        }
                    }
                }
            }
        }
    }
}

impl Drop for FolderContentAligner {
    fn drop(&mut self) {
        if std::thread::panicking() {
            return;
        }
        match self.folder_mode {
            AlignerFolderMode::ExpectNoOtherContent => self.verify_no_extra_content_exists(),
            AlignerFolderMode::ExpectOtherContent => {}
        }
    }
}
