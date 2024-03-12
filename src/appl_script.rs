use std::process::{Command, Output};

#[allow(dead_code)]
pub fn get_selected_text_from_clipboard() -> Result<String, std::io::Error> {
    let output = Command::new("osascript")
        .arg("-e")
        .arg("get the clipboard as text")
        .output()?;

    if output.status.success() {
        let selected_text = String::from_utf8_lossy(&output.stdout).to_string();
        Ok(selected_text)
    } else {
        Err(std::io::Error::new(
            std::io::ErrorKind::Other,
            "Failed to get selected text",
        ))
    }
}

pub fn get_selected_text() -> Result<String, std::io::Error> {
    let paste_script = r#"
        -- Back up clipboard contents:
        set savedClipboard to the clipboard

        -- Copy selected text to clipboard:
        tell application "System Events" to keystroke "c" using {command down}
        delay 1 -- Without this, the clipboard may have stale data.

        set theSelectedText to the clipboard

        -- Restore clipboard:
        set the clipboard to savedClipboard

        return theSelectedText
    "#;

    match run_applescript(paste_script) {
        Ok(output) => {
            if output.status.success() {
                let selected_text = String::from_utf8_lossy(&output.stdout).to_string();
                // println!("Selected Text: {}", selected_text);
                Ok(selected_text)
            } else {
                Err(std::io::Error::new(
                    std::io::ErrorKind::Other,
                    "Failed to get selected text",
                ))
            }
        }
        Err(err) => Err(err),
    }
}

fn run_applescript(script: &str) -> Result<Output, std::io::Error> {
    Command::new("osascript")
        .arg("-e")
        .arg(script)
        .output()
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn selected_t() {
        match get_selected_text() {
            Ok(text) => {
                println!("Selected Text: {}", text);
            }
            Err(err) => {
                eprintln!("Error: {}", err);
            }
        }
    }
}
