use kinode_process_lib::vfs::{open_dir, open_file, DirEntry, FileType, VfsAction, VfsRequest};
use kinode_process_lib::{println, Request};
use std::collections::HashMap;

pub mod encryption;
pub mod structs;

// outputs file contents
pub fn read_file(dir: DirEntry) -> anyhow::Result<String> {
    if dir.path.ends_with(".DS_Store") {
        return Err(anyhow::Error::msg("Skipping .DS_Store"));
    }
    let file = open_file(&dir.path, false, Some(5));
    let contents: Vec<u8> = file?.read()?;
    let json = std::str::from_utf8(&contents).map(|s| s.to_string())?;
    Ok(json)
}

// outputs map(path->contents) from a vector of files
pub fn read_files(dirs: Vec<DirEntry>) -> anyhow::Result<HashMap<String, String>> {
    let mut files = HashMap::new();
    for dir in dirs {
        let content = read_file(DirEntry {
            path: dir.path.to_string(),
            file_type: dir.file_type,
        });
        if let Ok(content) = content {
            files.insert(dir.path.to_string(), content);
        }
    }
    Ok(files)
}

// outputs map(path ->empty_contents)
pub fn read_files_light(dirs: Vec<DirEntry>) -> anyhow::Result<HashMap<String, String>> {
    let mut files = HashMap::new();
    for dir in dirs {
        files.insert(dir.path.to_string(), String::from(""));
    }
    Ok(files)
}

// outputs vec of paths from a dir
pub fn read_dir(dir: DirEntry) -> anyhow::Result<Vec<DirEntry>> {
    // println!("fn read_dir on: {:#?}", &dir);
    let dir = open_dir(&dir.path, false, Some(5));
    match dir {
        Ok(dir) => Ok(dir.read()?),
        Err(_) => Err(anyhow::Error::msg("Failed to read directory")),
    }
}

// outputs map(path ->contents) from arbitrary nested directory
pub fn read_nested_dir(dir: DirEntry) -> anyhow::Result<HashMap<String, String>> {
    //  read dir -> list of paths
    let entries = read_dir(dir)?;
    //  split files from dirs
    let (directories, files): (Vec<DirEntry>, Vec<DirEntry>) = entries
        .into_iter()
        .partition(|entry| entry.file_type == FileType::Directory);

    let mut output: HashMap<String, String> = HashMap::new();
    //  files -> read files -> map path contents
    output.extend(read_files(files)?);

    //  dirs ->
    //    for each -> read nested dir -> accumulate map path contents
    for dir in directories {
        output.extend(read_nested_dir(dir)?);
    }

    Ok(output)
}

// outputs map(path ->empty_contents) from arbitrary nested directory
pub fn read_nested_dir_light(dir: DirEntry) -> anyhow::Result<HashMap<String, String>> {
    //  read dir -> list of paths
    let entries = read_dir(dir)?;
    //  split files from dirs
    let (directories, files): (Vec<DirEntry>, Vec<DirEntry>) = entries
        .into_iter()
        .partition(|entry| entry.file_type == FileType::Directory);

    let mut output: HashMap<String, String> = HashMap::new();
    //  files -> read files -> map path contents
    output.extend(read_files_light(files)?);

    //  dirs ->
    //    for each -> read nested dir -> accumulate map path contents
    for dir in directories {
        output.extend(read_nested_dir_light(dir)?);
    }

    Ok(output)
}

// takes flattened directory contents and imports it to given directory
pub fn import_notes(directory: HashMap<String, String>, import_to: &String) -> anyhow::Result<()> {
    let mut dirs_created: Vec<String> = Vec::new();

    for (file_path, content) in directory.iter() {
        let full_file_path = format!("{}/{}", import_to, file_path);

        let mut split_path: Vec<&str> = full_file_path
            .split("/")
            .filter(|s| !s.is_empty())
            .collect::<Vec<&str>>();
        split_path.pop();
        let parent_path = split_path.join("/");

        // not perfect, i.e. it will run create /this/dir even if /this/dir/here/ exists
        // because it doesnt check what was already created when /this/dir/here was created
        if !dirs_created.contains(&parent_path) {
            // println!("creating dir: {:?}", dir_path);
            let request = VfsRequest {
                path: format!("/{}", parent_path).to_string(),
                action: VfsAction::CreateDirAll,
            };
            let _message = Request::new()
                .target(("our", "vfs", "distro", "sys"))
                .body(serde_json::to_vec(&request)?)
                .send_and_await_response(5)?;
        }

        dirs_created.push(parent_path);

        let file = open_file(&full_file_path, true, Some(5))?;
        file.write(content.as_bytes())?;
    }

    println!("done importing notes");
    Ok(())
}
