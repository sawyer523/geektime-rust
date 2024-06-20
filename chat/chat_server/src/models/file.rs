use std::path::{Path, PathBuf};
use std::str::FromStr;

use sha1::{Digest, Sha1};

use crate::{AppError, ChatFile};

impl ChatFile {
    pub fn new(ws_id: u64, filename: &str, data: &[u8]) -> Self {
        let hash = Sha1::digest(data);
        Self {
            ws_id,
            ext: filename.split(".").last().unwrap_or("txt").to_string(),
            hash: hex::encode(hash),
        }
    }

    pub fn url(&self) -> String {
        format!("/files/{}", self.hash_to_path())
    }

    pub fn path(&self, base_dir: &Path) -> PathBuf {
        base_dir.join(self.hash_to_path())
    }

    // split hash into 3 parts, first 2 with 3 chars
    fn hash_to_path(&self) -> String {
        let (part1, part2) = self.hash.split_at(3);
        let (part2, part3) = part2.split_at(3);
        format!("{}/{}/{}/{}.{}", self.ws_id, part1, part2, part3, self.ext)
    }
}

impl FromStr for ChatFile {
    type Err = AppError;

    // convert /files/1/c35/419/65321acb48b82be537efafba581c962099.yml to ChatFile
    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let Some(s) = s.strip_prefix("/files/") else {
            return Err(AppError::ChatFileError(format!(
                "invalid chat file path: {s}"
            )));
        };

        let parts: Vec<&str> = s.split('/').collect();
        if parts.len() != 4 {
            return Err(AppError::ChatFileError(format!(
                "file path {s} does not valid"
            )));
        }
        let Ok(ws_id) = parts[0].parse::<u64>() else {
            return Err(AppError::ChatFileError(format!(
                "invalid workspace id: {}",
                parts[0]
            )));
        };
        let Some((part3, ext)) = parts[3].rsplit_once('.') else {
            return Err(AppError::ChatFileError(
                "file name does not valid".to_string(),
            ));
        };
        let hash = format!("{}{}{}", parts[1], parts[2], part3);
        Ok(Self {
            ws_id,
            ext: ext.to_string(),
            hash,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn chat_file_new_should_work() {
        let file = ChatFile::new(1, "test.txt", b"test");
        assert_eq!(file.ws_id, 1);
        assert_eq!(file.ext, "txt");
        assert_eq!(file.hash, "a94a8fe5ccb19ba61c4c0873d391e987982fbbd3");
    }
}
