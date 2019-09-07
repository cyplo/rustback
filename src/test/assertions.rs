use std::fs;
use std::fs::File;
use std::io::Read;
use std::path::Path;

use tempfile::tempdir;
use walkdir::WalkDir;

use crate::error::BakareError;
use crate::repository::{ItemVersion, Repository};
use crate::source::TempSource;
use crate::{backup, restore};

pub fn assert_target_file_contents(restored_path: &Path, expected_contents: &str) -> Result<(), BakareError> {
    let mut actual_contents = String::new();
    assert!(restored_path.exists(), "Expected '{}' to be there", restored_path.display());
    File::open(restored_path)?.read_to_string(&mut actual_contents)?;
    assert_eq!(expected_contents, actual_contents);
    Ok(())
}

pub fn assert_same_after_restore(source_path: &Path) -> Result<(), BakareError> {
    let repository_path = tempdir()?.into_path();
    let restore_target = tempdir()?.into_path();

    assert_ne!(source_path, repository_path);
    assert_ne!(repository_path, restore_target);

    Repository::init(repository_path.as_path())?;
    {
        let mut backup_repository = Repository::open(repository_path.as_path())?;
        let mut backup_engine = backup::Engine::new(source_path, &mut backup_repository)?;
        backup_engine.backup()?;
    }
    {
        let restore_repository = Repository::open(repository_path.as_path())?;
        let restore_engine = restore::Engine::new(&restore_repository, &restore_target)?;
        restore_engine.restore_all()?;
    }

    assert_directory_trees_have_same_contents(source_path, restore_target.as_path())?;
    Ok(())
}

pub fn assert_restored_from_version_has_contents(
    repository_path: &Path,
    source_file_full_path: &Path,
    old_contents: &str,
    old_version: &ItemVersion,
) -> Result<(), BakareError> {
    let restore_repository = Repository::open(repository_path)?;
    let restore_target = tempdir()?;
    let restore_engine = restore::Engine::new(&restore_repository, &restore_target.path())?;
    let old_item = restore_repository.item_by_source_path_and_version(&source_file_full_path, &old_version)?;
    restore_engine.restore(&old_item.unwrap())?;
    let restored_file_path = restore_target.path().join(source_file_full_path.strip_prefix("/")?);
    assert_target_file_contents(&restored_file_path, old_contents)
}

pub fn item_version(repository_path: &Path, source_file_full_path: &Path) -> Result<ItemVersion, BakareError> {
    let old_version = {
        let reading_repository = Repository::open(repository_path)?;
        let item = reading_repository.item_by_source_path(&source_file_full_path)?;
        assert!(item.is_some());
        let item = item.unwrap();
        item.version().clone()
    };
    Ok(old_version)
}

pub fn read_restored_file_contents(
    source: TempSource,
    restore_target: &Path,
    source_file_relative_path: &str,
) -> Result<String, BakareError> {
    let source_file_full_path = source.file_path(source_file_relative_path);
    let restored_file_path = restore_target.join(source_file_full_path.strip_prefix("/")?);
    let contents = fs::read_to_string(restored_file_path)?;
    Ok(contents)
}

pub fn restore_all_from_reloaded_repository(repository_path: &Path, restore_target: &Path) -> Result<(), BakareError> {
    {
        let restore_repository = Repository::open(repository_path)?;
        let restore_engine = restore::Engine::new(&restore_repository, &restore_target)?;
        restore_engine.restore_all()?;
        Ok(())
    }
}

pub fn backup_file_with_contents(
    source: &TempSource,
    repository_path: &Path,
    source_file_relative_path: &str,
    contents: &str,
) -> Result<(), BakareError> {
    {
        let mut backup_repository = Repository::open(repository_path)?;
        let mut backup_engine = backup::Engine::new(source.path(), &mut backup_repository)?;
        source.write_text_to_file(source_file_relative_path, contents)?;
        backup_engine.backup()?;
        Ok(())
    }
}

fn assert_directory_trees_have_same_contents(left: &Path, right: &Path) -> Result<(), BakareError> {
    let left_files = get_sorted_files_recursively(left)?;
    let right_files = get_sorted_files_recursively(right)?;

    let pairs = left_files.iter().zip(right_files);
    for (l, r) in pairs {
        assert_eq!(l.file_name(), r.file_name());
        let mut fl = File::open(l)?;
        let mut fr = File::open(r)?;
        let mut bl = vec![];
        let mut br = vec![];
        fl.read_to_end(&mut bl)?;
        fr.read_to_end(&mut br)?;
        assert_eq!(bl, br);
    }
    Ok(())
}

fn get_sorted_files_recursively(path: &Path) -> Result<Vec<Box<Path>>, BakareError> {
    let walker = WalkDir::new(path).sort_by(|a, b| a.file_name().cmp(b.file_name()));

    let mut result = vec![];

    for maybe_entry in walker {
        let entry = maybe_entry?;
        if entry.path() == path {
            continue;
        }
        if entry.path().is_file() {
            result.push(Box::from(entry.path()));
        }
    }

    Ok(result)
}