//! Native dialogs run outside the render loop. No shell interprets their arguments.
use std::{
    ffi::OsString,
    io::{self, Read},
    os::unix::ffi::OsStringExt,
    path::PathBuf,
    process::{Child, Command, Stdio},
};

pub struct Picker {
    child: Child,
}

impl Picker {
    pub fn open() -> Result<Self, String> {
        let kde = [
            "--getopenfilename",
            ".",
            "*.sce|KovaaK scenario (*.sce)",
            "--title",
            "Import scenario",
        ];
        let gtk = [
            "--file-selection",
            "--title=Import scenario",
            "--file-filter=KovaaK scenario | *.sce",
        ];
        for (program, arguments) in [("kdialog", kde.as_slice()), ("zenity", gtk.as_slice())] {
            match Command::new(program)
                .args(arguments)
                .stdin(Stdio::null())
                .stdout(Stdio::piped())
                .stderr(Stdio::null())
                .spawn()
            {
                Ok(child) => return Ok(Self { child }),
                Err(error) if error.kind() == io::ErrorKind::NotFound => continue,
                Err(error) => return Err(format!("Cannot open file picker: {error}")),
            }
        }
        Err("File picker unavailable. Install kdialog or zenity, or use --import-scenario.".into())
    }

    pub fn poll(&mut self) -> Option<Result<Option<PathBuf>, String>> {
        let status = match self.child.try_wait() {
            Ok(None) => return None,
            Ok(Some(status)) => status,
            Err(error) => return Some(Err(format!("File picker failed: {error}"))),
        };
        if status.code() == Some(1) {
            return Some(Ok(None));
        }
        if !status.success() {
            return Some(Err(
                "File picker failed to open or exited unexpectedly".into()
            ));
        }
        let mut bytes = Vec::new();
        let result = self
            .child
            .stdout
            .take()
            .ok_or_else(|| "File picker returned no output".to_string())
            .and_then(|output| {
                output
                    .take(8193)
                    .read_to_end(&mut bytes)
                    .map_err(|error| format!("Cannot read selected file: {error}"))?;
                selected_path(bytes).map(Some)
            });
        Some(result)
    }
}

impl Drop for Picker {
    fn drop(&mut self) {
        // Closing the app also closes its pending dialog and reaps the child process.
        let _ = self.child.kill();
        let _ = self.child.wait();
    }
}

fn selected_path(mut bytes: Vec<u8>) -> Result<PathBuf, String> {
    if bytes.last() == Some(&b'\n') {
        bytes.pop();
    }
    if bytes.is_empty() || bytes.len() > 8192 || bytes.contains(&0) || bytes.contains(&b'\n') {
        return Err("File picker returned an invalid path".into());
    }
    let path = PathBuf::from(OsString::from_vec(bytes));
    if !path.is_absolute() {
        return Err("File picker must return a local absolute path".into());
    }
    Ok(path)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn dialog_cancel_and_empty_success_are_distinct() {
        for (program, cancelled) in [("false", true), ("true", false)] {
            let child = Command::new(program)
                .stdout(Stdio::piped())
                .spawn()
                .expect("test child");
            let mut picker = Picker { child };
            picker.child.wait().expect("child exit");
            let result = picker.poll().expect("completed child");
            if cancelled {
                assert_eq!(result.expect("cancel is not an error"), None);
            } else {
                assert!(result.is_err());
            }
        }
    }

    #[test]
    fn dialog_output_must_be_one_absolute_local_path() {
        assert_eq!(
            selected_path(b"/tmp/a scenario.sce\n".to_vec()).expect("path"),
            PathBuf::from("/tmp/a scenario.sce")
        );
        for bytes in [
            b"\n".to_vec(),
            b"relative.sce\n".to_vec(),
            b"/tmp/a\n/tmp/b\n".to_vec(),
            b"/tmp/a\0.sce".to_vec(),
            vec![b'a'; 8194],
        ] {
            assert!(selected_path(bytes).is_err());
        }
    }
}
