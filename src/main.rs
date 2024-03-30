mod path;
use std::{
    collections::{BTreeMap, HashMap},
    env,
    error::Error,
    fs,
    path::PathBuf,
};

use path::BackupFile;

// keep last N archives
const KEEP_LAST_N_ARCHIVES: usize = 5;
// keep last N monthly archives (the first archive of a month) in current year
// including current month
const KEEP_LAST_N_MONTHS: usize = 2;
// keep last N annually archives (the first archive of a year)
// including current year
const KEEP_LAST_N_YEARS: usize = 3;

fn main() -> Result<(), Box<dyn Error>> {
    let target_dir = env::args().nth(1).ok_or_else(|| {
        let my_path = env::args().nth(0).unwrap();
        format!("{my_path} <TARGET DIR>")
    })?;
    let target_dir = PathBuf::from(target_dir);

    target_dir
        .is_dir()
        .then_some(())
        .ok_or("target dir should be a directory")?;

    let mut target_backupfiles: HashMap<String, Vec<BackupFile>> = HashMap::new();
    for backup_file in target_dir
        .read_dir()?
        .filter_map(Result::ok)
        .map(|e| e.path())
        .filter_map(BackupFile::new)
    {
        target_backupfiles
            .entry(backup_file.target.clone())
            .or_default()
            .push(backup_file);
    }

    for backup_files in target_backupfiles.into_values() {
        let delete_list = backups_to_delete(backup_files.into_iter())?;
        for path in delete_list.map(|bf| bf.path) {
            if let Err(e) = fs::remove_file(&path) {
                println!(
                    "Error occured when deleting {}: {e}",
                    path.to_string_lossy()
                )
            }
        }
    }

    Ok(())
}

fn backups_to_delete(
    backup_files: impl Iterator<Item = BackupFile>,
) -> Result<impl Iterator<Item = BackupFile>, Box<dyn Error>> {
    // year -> backup files
    let mut backfile_map: BTreeMap<u32, Vec<BackupFile>> = BTreeMap::new();

    for (year, backup_file) in backup_files.map(|back| (back.year, back)) {
        backfile_map.entry(year).or_default().push(backup_file);
    }

    // sort backup files by timestamps in filename
    backfile_map.iter_mut().for_each(|(_, backup_files)| {
        backup_files.sort();
    });

    let current_year = backfile_map
        .keys()
        .last()
        .expect("no backup file was found")
        .clone();

    let current_month = backfile_map
        .entry(current_year)
        .or_default()
        .last()
        .unwrap()
        .month;

    // keep every first backup of a year,
    // keep at most `KEEP_LAST_N_MONTHS` annually backup
    backfile_map
        .iter_mut()
        .rev()
        .take(KEEP_LAST_N_YEARS)
        .for_each(|(_, b)| {
            b[0].keep = true;
        });

    // keep latest 2 archives of at most KEEP_LAST_N_MONTHS
    // previous months in this year. only keep the first archive
    // of a month
    backfile_map.entry(current_year).and_modify(|b| {
        let mut month_seen: BTreeMap<_, &mut BackupFile> = BTreeMap::new();
        for backup_file in b.iter_mut().rev() {
            // for every month only the first archive will be kept,
            // as `insert`` will replace previous value.
            month_seen.insert(backup_file.month, backup_file);
        }

        for (_, backup_file) in month_seen.into_iter().rev().take(KEEP_LAST_N_MONTHS) {
            backup_file.keep = true;
        }
    });

    // keep every archive of this month
    backfile_map.entry(current_year).and_modify(|b| {
        b.iter_mut()
            .filter(|b| b.month == current_month)
            .for_each(|b| {
                b.keep = true;
            });
    });

    Ok(backfile_map
        .into_iter()
        .map(|(_, files)| files.into_iter())
        .flatten()
        .rev()
        .skip(KEEP_LAST_N_ARCHIVES) // last N archives that always keep
        .filter(|b| b.keep == false))
}
