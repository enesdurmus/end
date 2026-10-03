//! General pasteboard reads and writes.

use objc2::rc::Retained;
use objc2::runtime::ProtocolObject;
use objc2_app_kit::NSPasteboard;
use objc2_foundation::{NSArray, NSString, NSURL};

pub(super) fn copy_files(paths: &[String]) -> Result<(), String> {
    let urls: Vec<Retained<NSURL>> =
        paths.iter().map(|p| NSURL::fileURLWithPath(&NSString::from_str(p))).collect();
    let objs: Vec<_> = urls.iter().map(|u| ProtocolObject::from_ref(&**u)).collect();
    let pb = NSPasteboard::generalPasteboard();
    pb.clearContents();
    if pb.writeObjects(&NSArray::from_slice(&objs)) {
        Ok(())
    } else {
        Err("clipboard rejected the files".into())
    }
}

pub(super) fn files() -> Vec<String> {
    let pb = NSPasteboard::generalPasteboard();
    let Some(items) = pb.pasteboardItems() else {
        return Vec::new();
    };
    let ty = NSString::from_str("public.file-url");
    items
        .iter()
        .filter_map(|item| item.stringForType(&ty))
        .filter_map(|s| NSURL::URLWithString(&s))
        .filter_map(|url| url.path().map(|p| p.to_string()))
        .collect()
}

pub(super) fn source_url() -> Option<String> {
    let pb = NSPasteboard::generalPasteboard();
    let s = pb.stringForType(&NSString::from_str("public.url"))?.to_string();
    (!s.is_empty()).then_some(s)
}

pub(super) fn change_count() -> Option<u64> {
    Some(NSPasteboard::generalPasteboard().changeCount() as u64)
}

#[cfg(test)]
mod tests {
    use crate::platform::{macos::MacOs, Platform};

    /// Overwrites the real clipboard, so it is ignored by default: run with
    /// `cargo test -- --ignored`. One test rather than three, because the general
    /// pasteboard is global state and cargo runs tests in parallel.
    #[test]
    #[ignore]
    fn files_round_trip_through_the_pasteboard() {
        use objc2_app_kit::NSPasteboard;
        use objc2_foundation::NSString;

        MacOs.copy_files(&["/tmp/launcher-copy-file-test.gif".to_string()]).unwrap();
        let pb = NSPasteboard::generalPasteboard();
        let got = pb.stringForType(&NSString::from_str("public.file-url"));
        assert_eq!(
            got.map(|s| s.to_string()).as_deref(),
            Some("file:///tmp/launcher-copy-file-test.gif")
        );

        let paths = vec![
            "/tmp/launcher clip test/a b.txt".to_string(),
            "/tmp/launcher clip test/c.txt".to_string(),
        ];
        let before = MacOs.clipboard_change_count().unwrap();
        MacOs.copy_files(&paths).unwrap();
        assert_eq!(MacOs.clipboard_files(), paths, "percent-escapes must be decoded");
        assert!(MacOs.clipboard_change_count().unwrap() > before);

        arboard::Clipboard::new().unwrap().set_text("just text").unwrap();
        assert!(MacOs.clipboard_files().is_empty(), "plain text is not a file list");
    }
}
