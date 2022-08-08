use std::path::{Path, PathBuf};

use chrono::NaiveDateTime;
use xshell::{cmd, Shell};

const INPUT_DATE_FMT: &str = "%Y-%m-%d %H:%M:%S";
const OUTPUT_DATE_FMT: &str = "%Y-%m-%dT%H:%M:%S";

#[derive(Debug)]
pub struct Metadata {
    artist: String,
    date: NaiveDateTime,
    extension: String,
}

impl Metadata {
    pub fn exiftool<P: AsRef<Path>>(sh: &Shell, path: &P) -> anyhow::Result<Metadata> {
        let path = path.as_ref();
        let stdout = cmd!(
            sh,
            "exiftool -d {INPUT_DATE_FMT} -createdate -artist -T {path}"
        )
        .read()?;
        let mut iter = stdout.split_terminator('\t').map(String::from);
        let date = iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse date"))?;
        let artist = iter
            .next()
            .ok_or_else(|| anyhow::anyhow!("Failed to parse artist"))?;
        let metadata = Metadata {
            artist,
            date: NaiveDateTime::parse_from_str(&date, INPUT_DATE_FMT)?,
            extension: path.extension().unwrap().to_string_lossy().to_string(),
        };
        Ok(metadata)
    }

    fn initials(&self) -> String {
        self.artist
            .split_terminator(' ')
            .flat_map(|word| word.chars().next())
            .collect()
    }

    pub fn new_file_name(&self, destination: &Path) -> PathBuf {
        assert!(destination.is_dir(), "Expected a directory for destination");
        assert!(
            destination.is_absolute(),
            "Expected an absolute path for destination"
        );

        let mut file_name = String::new();
        let date = format!("{}", self.date.format(OUTPUT_DATE_FMT));
        file_name.push_str(&self.initials());
        file_name.push('-');
        file_name.push_str(&date);
        file_name.push('.');
        file_name.push_str(&self.extension);

        destination.join(file_name)
    }
}
