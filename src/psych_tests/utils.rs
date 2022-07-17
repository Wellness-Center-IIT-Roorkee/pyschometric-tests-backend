use rand::distributions::Alphanumeric;
use rand::{thread_rng, Rng};
use rocket::fs::TempFile;
use std::env;
use std::error::Error;
use std::path::Path;

pub async fn unsafe_save_file(mut file: TempFile<'_>) -> Result<String, Box<dyn Error>> {
    let media_file_dir = env::var("MEDIA_FILES_DIR").unwrap();
    let mut name: String = thread_rng()
        .sample_iter(&Alphanumeric)
        .take(50)
        .map(char::from)
        .collect();

    // TODO: Use a safe method to determine file extension
    let (_, extension) = file
        .raw_name()
        .unwrap()
        .dangerous_unsafe_unsanitized_raw()
        .as_str()
        .rsplit_once('.')
        .unwrap();
    name.push('.');
    name.push_str(extension);

    file.persist_to(Path::new(&media_file_dir).join("test_logo").join(name))
        .await?;

    Ok(file.path().unwrap().to_string_lossy().to_string())
}
