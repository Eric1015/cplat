use clap::{crate_authors, crate_version, Arg, Command};
use std::{fs, cmp::Ordering, path::Path};
use directories::UserDirs;
use exitcode;

fn main() {
    let matches = cli().get_matches();
    let name = matches.get_one("name");
    let destination = matches.get_one("destination");

    if let Some(user_dirs) = UserDirs::new() {
        let download_dir_path = user_dirs.download_dir().unwrap();
        // Linux:   /home/alice/Downloads
        // Windows: C:\Users\Alice\Downloads
        // macOS:   /Users/Alice/Downloads

        let mut files = get_all_files_in_dir(download_dir_path);

        let latest_file = get_latest_file(&mut files);

        copy_file_to_destination(latest_file, name, destination)
    } else {
        eprint!("Failed to copy the latest file. No Downloads folder found");
        std::process::exit(exitcode::CONFIG);
    }
}

fn cli() -> Command {
    Command::new("cplat")
        .author(crate_authors!())
        .version(crate_version!())
        .about("Copy the latest file in the download folder to where you are now!")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .value_name("NAME")
                .help("Rename of the file after being copied to destination. Default to its original name if not sepcified")
        )
        .arg(
            Arg::new("destination")
                .short('d')
                .long("destination")
                .value_name("PATH_TO_DESTINATION")
                .help("Path to where the file should be copied to. Default to where this command is executed if not specified")
        )
}

fn get_all_files_in_dir(dir_path: &Path) -> Vec<fs::DirEntry> {
    let Ok(entries) = fs::read_dir(dir_path) else { 
        eprint!("Failed to read files in Downloads folder. Terminating the process.");
        std::process::exit(exitcode::IOERR);
     };
    let files: Vec<fs::DirEntry> = entries.flatten().filter(|entry| { 
        let Ok(meta) = entry.metadata() else {
            eprint!("Failed to get metadata for file: {:?}, skipping this file.", entry.file_name());
            return false;
        };
        return meta.is_file();
    }).collect();

    return files;
}

fn get_latest_file(files: &mut Vec<fs::DirEntry>) -> &fs::DirEntry {
    files.sort_by(|a, b| {
        let Ok(m1) = a.metadata() else { return Ordering::Less };
        let Ok(m2) = b.metadata() else { return Ordering::Greater };
        let Ok(created1) = m1.created() else { return Ordering::Less };
        let Ok(created2) = m2.created() else { return Ordering::Greater };
        return created1.cmp(&created2);
    });

    files.reverse();

    let Some(latest_file) = files.get(0) else { 
        eprint!("No files found in Downloads folder. Terminating the process.");
        std::process::exit(exitcode::CONFIG);
    };

    return latest_file;
}

fn copy_file_to_destination(file: &fs::DirEntry, name: Option<&String>, destination: Option<&String>) {
    let original_file_name = file.file_name().into_string().unwrap();
    let file_path = file.path();
    let result_file_name = String::from(name.unwrap_or(&original_file_name));
    let destination_path = String::from(destination.unwrap_or(&".".to_owned()));
    let destination_file_path = format!("{}/{}", &destination_path, &result_file_name);
    match fs::copy(file_path.clone(), destination_file_path.clone()) {
        Ok(_) => {
            print!("Successfully copied the latest file: {} to {}", &original_file_name, &destination_file_path);
            std::process::exit(exitcode::OK);
        },
        Err(err) => {
            eprint!("Failed to copy the latest file: {} to {} due to error: {}", &original_file_name, &destination_file_path, err);
            std::process::exit(exitcode::IOERR);
        }
    };
}
