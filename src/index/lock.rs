use std::fs::File;
use std::path::Path;
use std::time::Duration;
use std::{fs, thread};

use uuid::Uuid;

use glob::{glob, Paths};

use crate::error::BakareError;

pub fn release_lock(path: &Path, lock_id: Uuid) -> Result<(), BakareError> {
    let lock_file_path = lock_file_path(path, lock_id);
    delete_lock_file(lock_file_path)?;
    Ok(())
}

fn delete_lock_file(lock_file_path: String) -> Result<(), BakareError> {
    fs::remove_file(lock_file_path.clone()).map_err(|e| (e, lock_file_path))?;
    Ok(())
}

pub fn acquire_lock(lock_id: Uuid, index_directory: &Path) -> Result<(), BakareError> {
    let lock_file_path = lock_file_path(index_directory, lock_id);
    create_lock_file(lock_file_path.clone())?;
    Ok(())
}

pub fn sole_lock(lock_id: Uuid, index_directory: &Path) -> Result<bool, BakareError> {
    let my_lock_file_path = lock_file_path(index_directory, lock_id);
    let mut locks = all_locks(index_directory)?;
    let only_my_locks = locks.all(|path| match path {
        Ok(path) => path.to_string_lossy() == my_lock_file_path,
        Err(_) => false,
    });
    Ok(only_my_locks)
}

fn all_locks(index_directory: &Path) -> Result<Paths, BakareError> {
    Ok(glob(&locks_glob(index_directory))?)
}

fn create_lock_file<T>(lock_file_path: T) -> Result<(), BakareError>
where
    T: AsRef<Path>,
{
    File::create(&lock_file_path).map_err(|e| (e, &lock_file_path))?;
    Ok(())
}

fn lock_file_path(path: &Path, lock_id: Uuid) -> String {
    format!("{}/{}.lock", path.to_string_lossy(), lock_id)
}

fn locks_glob(path: &Path) -> String {
    format!("{}/*.lock", path.to_string_lossy())
}
