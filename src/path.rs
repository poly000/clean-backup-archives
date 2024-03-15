use std::path::PathBuf;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct BackupFile {
    pub path: PathBuf,
    pub year: u32,
    pub month: u32,
    pub full_time: String,
    pub keep: bool,
    pub target: String,
}

impl PartialOrd for BackupFile {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        self.full_time.partial_cmp(&other.full_time)
    }
}

impl Ord for BackupFile {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        self.full_time.cmp(&other.full_time)
    }
}

impl BackupFile {
    pub fn new(path: PathBuf) -> Option<BackupFile> {
        let target = path
            .file_name()?
            .to_string_lossy()
            .split('.')
            .next()?
            .split_once("target_")?
            .1
            .to_string();

        let full_time = path.file_name()?.to_string_lossy().split('.').last()?[1..].to_string();
        let (yyyymmdd, _hhmmss) = full_time.split_once('-')?;

        let year = yyyymmdd[..4].parse().ok()?;
        let month = yyyymmdd[4..6].parse().ok()?;
        Some(BackupFile {
            target,
            path,
            year,
            month,
            full_time,
            keep: false,
        })
    }
}
