use cosmic::iced::clipboard::mime::AllowedMimeTypes;
use std::{borrow::Cow, error::Error, path::PathBuf, str};
use url::Url;

#[derive(Clone, Debug)]
pub struct DndDrop {
    pub paths: Vec<PathBuf>,
}

impl AllowedMimeTypes for DndDrop {
    fn allowed() -> Cow<'static, [String]> {
        Cow::from(vec![
            "x-special/gnome-copied-files".to_string(),
            "text/uri-list".to_string(),
        ])
    }
}

impl TryFrom<(Vec<u8>, String)> for DndDrop {
    type Error = Box<dyn Error>;
    fn try_from(value: (Vec<u8>, String)) -> Result<Self, Self::Error> {
        let (data, mime) = value;
        let mut paths = Vec::new();
        match mime.as_str() {
            "text/uri-list" => {
                let text = str::from_utf8(&data)?;
                for line in text.lines() {
                    let url = Url::parse(line)?;
                    match url.to_file_path() {
                        Ok(path) => paths.push(path),
                        Err(()) => Err(format!("invalid file URL {:?}", url))?,
                    }
                }
            }
            "x-special/gnome-copied-files" => {
                let text = str::from_utf8(&data)?;
                for (i, line) in text.lines().enumerate() {
                    if i != 0 {
                        let url = Url::parse(line)?;
                        match url.to_file_path() {
                            Ok(path) => paths.push(path),
                            Err(()) => Err(format!("invalid file URL {:?}", url))?,
                        }
                    }
                }
            }
            _ => Err(format!("unsupported mime type {:?}", mime))?,
        }
        Ok(Self { paths })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn parses_text_uri_list() {
        let data = b"file:///home/user/a.txt\nfile:///home/user/b.txt\n".to_vec();
        let dnd: DndDrop = DndDrop::try_from((data, "text/uri-list".to_string())).unwrap();
        assert_eq!(dnd.paths.len(), 2);
        assert_eq!(dnd.paths[0], PathBuf::from("/home/user/a.txt"));
        assert_eq!(dnd.paths[1], PathBuf::from("/home/user/b.txt"));
    }

    #[test]
    fn parses_gnome_copied_files_skipping_action_line() {
        let data = b"copy\nfile:///home/user/c.txt\nfile:///home/user/d.txt\n".to_vec();
        let dnd: DndDrop =
            DndDrop::try_from((data, "x-special/gnome-copied-files".to_string())).unwrap();
        assert_eq!(dnd.paths.len(), 2);
        assert_eq!(dnd.paths[0], PathBuf::from("/home/user/c.txt"));
        assert_eq!(dnd.paths[1], PathBuf::from("/home/user/d.txt"));
    }

    #[test]
    fn rejects_invalid_file_url() {
        let data = b"file:///home/user/ok.txt\nhttp://not-a-file-path\n".to_vec();
        let result = DndDrop::try_from((data, "text/uri-list".to_string()));
        assert!(result.is_err());
    }

    #[test]
    fn rejects_unknown_mime_type() {
        let data = b"file:///home/user/e.txt\n".to_vec();
        let result = DndDrop::try_from((data, "application/octet-stream".to_string()));
        assert!(result.is_err());
    }
}
