use regex::{Match, Regex};
use std::{
    cell::RefCell,
    fs::{self, DirEntry},
    io,
    path::Path,
    thread,
};

fn main() -> io::Result<()> {
    let pending_threads = RefCell::new(Vec::new());

    visit_acs_files(&Path::new("."), &|entry| {
        let path = entry.path();
        pending_threads.borrow_mut().push(thread::spawn(move || {
            let cell_delimiters = Regex::new(r#"(?mi)(?:(?:[a-z]+\d*)|")(\.)[a-z]+"#).unwrap();
            let member_fetch = Regex::new(r#"(?mi)(?:(?:[a-z]\d*)|}|\))(:)[a-z]"#).unwrap();
            let sourced_address = Regex::new(r#"(?mi)(?: |"|\(|\[)(!)[a-z]"#).unwrap();
            let namespace = Regex::new(r#"(?mi)[a-z]\d*(#)[a-z]"#).unwrap();

            println!("Reading {:?}", path);
            let mut content = fs::read_to_string(&path).unwrap();
            let original = content.clone();

            for captures in cell_delimiters.captures_iter(&original) {
                let capture: Match = captures.get(1).unwrap();
                content.replace_range(capture.start()..capture.end(), ":");
            }

            for captures in member_fetch.captures_iter(&original) {
                let capture: Match = captures.get(1).unwrap();
                content.replace_range(capture.start()..capture.end(), ".");
            }

            for captures in sourced_address.captures_iter(&original) {
                let capture: Match = captures.get(1).unwrap();
                content.replace_range(capture.start()..capture.end(), "~");
            }

            // Reverse because we're adding characters
            for captures in namespace
                .captures_iter(&original)
                .collect::<Vec<_>>()
                .iter()
                .rev()
            {
                let capture: Match = captures.get(1).unwrap();
                content.replace_range(capture.start()..capture.end(), "::");
            }

            println!("Old content:\n{}", original);
            println!("New content:\n{}", content);

            fs::write(String::from(path.to_str().unwrap()) + ".new", content).expect("Oh noes");
        }));
    })?;

    for handle in pending_threads.borrow_mut().drain(..) {
        handle.join().unwrap();
    }

    Ok(())
}

fn visit_acs_files(dir: &Path, cb: &dyn Fn(&DirEntry)) -> io::Result<()> {
    if dir.is_file() {
        return Ok(());
    }

    for entry in fs::read_dir(dir)? {
        let entry = entry?;
        let path = entry.path();
        let is_file = path.to_str().unwrap().ends_with(&".acs");
        if path.is_dir() {
            visit_acs_files(&path, cb)?;
        } else if is_file {
            cb(&entry);
        }
    }

    Ok(())
}
