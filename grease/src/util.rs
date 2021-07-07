//! Extra utilties for use elsewhere in the API.

use base64::decode;
use chrono::{Local, NaiveDateTime};
use error::{GreaseError, GreaseResult};
use glob::glob;
use serde::Deserialize;
use std::collections::HashMap;
use std::fs::OpenOptions;
use std::io::Write;
use std::path::PathBuf;
use std::process::{Command, Stdio};
use std::str::FromStr;

pub struct Email {
    pub to_address: String,
    pub subject: String,
    pub content: String,
}

impl Email {
    pub const DEFAULT_NAME: &'static str = "Glee Club Officers";
    pub const DEFAULT_ADDRESS: &'static str = "gleeclub_officers@lists.gatech.edu";

    pub fn send(&self) -> GreaseResult<()> {
        let mut mail = Command::new("mail")
            .args(&["-s", &self.subject, &self.to_address])
            .stdin(Stdio::piped())
            .spawn()
            .map_err(|err| {
                GreaseError::ServerError(format!("Couldn't run mail to send an email: {}", err))
            })?;
        mail.stdin
            .as_mut()
            .ok_or(GreaseError::ServerError(
                "No stdin was available for mail.".to_owned(),
            ))?
            .write_all(self.content.as_bytes())
            .map_err(|err| {
                GreaseError::ServerError(format!("Couldn't send an email with mail: {}", err))
            })?;
        let output = mail.wait_with_output().map_err(|err| {
            GreaseError::ServerError(format!(
                "The output of the mail command couldn't be retrieved: {}",
                err
            ))
        })?;

        if output.status.success() {
            Ok(())
        } else {
            let error_message = std::str::from_utf8(&output.stderr).map_err(|_err| {
                GreaseError::ServerError(
                    "mail errored out with a non-utf8 error message.".to_owned(),
                )
            })?;
            Err(GreaseError::ServerError(format!(
                "mail failed to send an email with error code {}: {}",
                output.status.code().unwrap_or(1),
                error_message
            )))
        }
    }
}

#[derive(Deserialize)]
pub struct FileUpload {
    pub path: String,
    pub content: String,
}

impl FileUpload {
    pub fn upload(&self) -> GreaseResult<()> {
        let content = decode(&self.content).map_err(|err| {
            GreaseError::BadRequest(format!("couldn't decode file as base64: {}", err))
        })?;
        let path = {
            let given_path = PathBuf::from_str(&self.path).map_err(|_err| {
                GreaseError::BadRequest(format!("invalid file name: {}", &self.path))
            })?;
            let file_name = given_path.file_name().ok_or(GreaseError::BadRequest(
                "file name must end in an absolute path".to_owned(),
            ))?;
            let _extension = given_path.extension().ok_or(GreaseError::BadRequest(
                "file must have an extension".to_owned(),
            ))?;
            let mut base_path = PathBuf::from("../httpdocs/music/");
            base_path.push(file_name);

            base_path
        };
        let mut file = OpenOptions::new()
            .create(true)
            .write(true)
            .open(path)
            .map_err(|err| GreaseError::ServerError(format!("error opening file: {}", err)))?;
        file.set_len(0).map_err(|err| {
            GreaseError::ServerError(format!("unable to clear file for link: {}", err))
        })?;
        file.write_all(&content)
            .map_err(|err| GreaseError::ServerError(format!("error writing to file: {}", err)))?;

        Ok(())
    }
}

pub fn check_for_music_file(path: &str) -> GreaseResult<String> {
    let given_path = PathBuf::from_str(path)
        .map_err(|_err| GreaseError::BadRequest(format!("invalid file name: {}", path)))?;
    let file_name = given_path
        .file_name()
        .ok_or(GreaseError::BadRequest(
            "file name must end in an absolute path".to_owned(),
        ))?
        .to_string_lossy()
        .to_string();
    let _extension = given_path.extension().ok_or(GreaseError::BadRequest(
        "file must have an extension".to_owned(),
    ))?;

    let existing_path = {
        let mut path = PathBuf::from("../httpdocs/music/");
        path.push(&file_name);
        path
    };

    if std::fs::metadata(existing_path).is_ok() {
        Ok(file_name)
    } else {
        Err(GreaseError::BadRequest(format!(
            "the file {} doesn't exist yet and must be uploaded before a link to it can be made",
            file_name
        )))
    }
}

pub fn log_panic(request: &cgi::Request, error_message: String) -> cgi::Response {
    let now = Local::now().naive_local();
    let file_name = format!("./log/log {}.txt", now.format("%c"));
    let mut file = OpenOptions::new()
        .create_new(true)
        .write(true)
        .open(file_name)
        .expect("couldn't open new log file");
    let write_to_file = |file: &mut std::fs::File, content: String| {
        file.write_all(content.as_bytes())
            .expect("couldn't write to log file");
    };

    let headers = request
        .headers()
        .into_iter()
        .map(|(key, value)| {
            (
                key.to_string(),
                value.to_str().unwrap_or_default().to_string(),
            )
        })
        .collect::<HashMap<String, String>>();
    write_to_file(
        &mut file,
        format!(
            "At {}, panicked during request handling.\n",
            now.format("%c")
        ),
    );
    write_to_file(&mut file, format!("Headers:\n  {:?}\n", headers));
    write_to_file(&mut file, format!("Request:\n  {:?}\n", request));
    write_to_file(
        &mut file,
        format!("Error generated:\n  {}\n", error_message),
    );

    clean_up_old_logs();

    let json_val = serde_json::json!({
        "message": "Panicked during handling of request. Please contact an administrator with the following information:",
        "time": now.format("%c").to_string(),
        "request": format!("{:?}", request),
        "error": error_message,
        "headers": headers,
    });
    let body = json_val.to_string().into_bytes();

    cgi::http::response::Builder::new()
        .status(500)
        .body(body)
        .unwrap()
}

fn clean_up_old_logs() {
    let log_files: Vec<PathBuf> = glob("./log/*.txt")
        .expect("Failed to read glob pattern")
        .collect::<Result<Vec<_>, _>>()
        .expect("one of the log files had an invalid name");
    if log_files.len() >= 50 {
        let mut log_times = log_files
            .iter()
            .map(|log_file: &PathBuf| {
                let file_name = log_file
                    .file_name()
                    .expect("no file name found for log file")
                    .to_string_lossy();
                let time = NaiveDateTime::parse_from_str(&file_name, "log %c")
                    .expect("log file was incorrectly named");
                (log_file, time)
            })
            .collect::<Vec<(&PathBuf, NaiveDateTime)>>();
        log_times.sort_by_key(|(_log_file, time)| time.clone());

        log_times.iter().skip(50).for_each(|(log_file, _time)| {
            std::fs::remove_file(log_file).expect("could not delete old log file");
        });
    }
}

pub fn write_zip_to_directory<D: AsRef<std::path::Path>>(
    zip_contents: &Vec<u8>,
    directory: D,
) -> GreaseResult<()> {
    use std::{fs, io, path};

    let mut archive = zip::ZipArchive::new(io::Cursor::new(zip_contents)).map_err(|err| {
        GreaseError::BadRequest(format!("Couldn't read zip as a zip archive: {}", err))
    })?;
    let base_path: &path::Path = directory.as_ref();

    for i in 0..archive.len() {
        let mut file = archive.by_index(i).unwrap();
        let outpath: PathBuf = base_path.join(
            file.sanitized_name()
                .into_iter()
                .skip(1)
                .collect::<PathBuf>(),
        );

        if (&*file.name()).ends_with('/') {
            fs::create_dir_all(&outpath).map_err(|err| {
                GreaseError::ServerError(format!("Couldn't create a directory: {}", err))
            })?;
        } else {
            if let Some(p) = outpath.parent() {
                if !p.exists() {
                    fs::create_dir_all(&p).map_err(|err| {
                        GreaseError::ServerError(format!("Couldn't create a directory: {}", err))
                    })?;
                }
            }
            let mut outfile = fs::File::create(&outpath).map_err(|err| {
                GreaseError::ServerError(format!("Couldn't create a file: {}", err))
            })?;
            io::copy(&mut file, &mut outfile).map_err(|err| {
                GreaseError::ServerError(format!("Couldn't write to a file: {}", err))
            })?;
        }
    }

    Ok(())
}
