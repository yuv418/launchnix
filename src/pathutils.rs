use std::env;
use std::path::PathBuf;

pub fn merge_exe_path(push_path: &str) -> PathBuf {
    // If this doesn't work, it's definitely appropriate to panic.

    let mut exe_path = env::current_exe()
        .expect("Your binary isn't in a directory?? Something very strange is happening.");

    exe_path.pop();
    exe_path.push(push_path);
    exe_path
}
