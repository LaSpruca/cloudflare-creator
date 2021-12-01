use std::{
    fs::remove_file,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use crate::error::{Error, ErrorKind};

static PROGRAM: &str = include_str!("main.go");

pub fn create_source_file(
    cf_token: String,
    cf_zone: String,
    cf_domain: String,
    cf_email: String,
) -> Result<String, Error> {
    let source = PROGRAM
        .replace("@token", &cf_token)
        .replace("@zone", &cf_zone)
        .replace("@email", &cf_email)
        .replace("@dns", &cf_domain);

    let filename = format!(
        "i-{}.go",
        SystemTime::now()
            .duration_since(UNIX_EPOCH)
            .unwrap()
            .as_micros()
    );

    match std::fs::write(&filename, source.as_bytes()) {
        Ok(_) => {}
        Err(err) => {
            return Err(Error::new(
                ErrorKind::IOError,
                format!("Error writing to source file {}", err),
            ))
        }
    };
    Ok(filename)
}

pub fn compile_source(path: &str) -> Result<String, Error> {
    let output_path = format!(
        "cf-update-{}",
        path.strip_prefix("i-")
            .unwrap()
            .strip_suffix(".go")
            .unwrap()
    );
    let mut cmd = Command::new("go")
        .arg("build")
        .arg("-ldflags")
        .arg("-s -w")
        .arg("-o")
        .arg(output_path.as_str())
        .arg(path)
        .spawn()
        .unwrap();

    let output = match cmd.wait() {
        Ok(a) => a,
        Err(err) => {
            return Err(Error::new(
                ErrorKind::CompilerError,
                format!("Compile command failed {}", err),
            ))
        }
    };

    if !output.success() {
        match remove_file(&path) {
            Ok(_) => {}
            Err(err) => {
                return Err(Error::new(ErrorKind::IOErrorNonFatal, format!("{}", err)));
            }
        }

        return Err(Error::new(
            ErrorKind::CompilerError,
            "Compilation exited with non-zero exit code".into(),
        ));
    };

    Ok(output_path)
}
