mod error;

use error::*;

use std::{
    fs::remove_file,
    io::{Read, Write},
    net::TcpStream,
    path::Path,
    process::Command,
    time::{SystemTime, UNIX_EPOCH},
};

use ssh2::Session;

static PROGRAM: &str = include_str!("main.go");

macro_rules! ssh_err {
    ($stmt:expr,$msg:expr) => {
        match $stmt {
            Ok(a) => a,
            Err(e) => return { Err(Error::new(ErrorKind::SSHError, format!("{} {}", $msg, e))) },
        }
    };
}

fn main() {
    let source_file_path = create_source_file(
        "XXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXXX".into(),
        "laspruca.nz".into(),
        "devtron.laspruca.nz".into(),
    )
    .unwrap();

    let compiled_path = compile_source(source_file_path.clone()).unwrap();

    upload_program(
        compiled_path.clone(),
        "".into(),
        22,
        "".into(),
        Some("".into()),
        None,
    )
    .unwrap();

    remove_file(&source_file_path).unwrap();
    remove_file(&compiled_path).unwrap();
}

fn create_source_file(
    cf_token: String,
    cf_zone: String,
    cf_domain: String,
) -> Result<String, Error> {
    let source = PROGRAM
        .replace("@token", &cf_token)
        .replace("@zone", &cf_zone)
        .replace("@domain", &cf_domain)
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

fn compile_source(path: String) -> Result<String, Error> {
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
        .arg(path.as_str())
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

#[allow(dead_code)]
fn upload_program(
    filename: String,
    host: String,
    port: usize,
    username: String,

    password: Option<String>,
    key: Option<String>,
) -> Result<(), Error> {
    // Make a TCP connection to the server
    let tcp_stream = TcpStream::connect(format!("{}:{}", &host, port)).unwrap();

    // Start a new SSH session
    let mut sess = ssh_err!(Session::new(), "Unable to create SSH session");
    sess.set_tcp_stream(tcp_stream);
    ssh_err!(sess.handshake(), "Unable to make ssh handshake");

    // Authenticate
    if let Some(pass) = password {
        ssh_err!(
            sess.userauth_password(&username, &pass),
            "Unable to authenticate with username/password"
        );
    } else if let Some(key) = key {
        ssh_err!(
            sess.userauth_pubkey_memory(&username, None, &key, None),
            "Unable to authenticate with pubkey"
        );
    }

    upload_file(&filename, &sess)?;

    let mut channel = ssh_err!(sess.channel_session(), "Error creating ssh channel");
    ssh_err!(
        channel.exec(&format!("chmod +x {}", &filename)),
        "Error running chmod"
    );

    ssh_err!(
        channel.exec("crontab -l > mycrontab"),
        "Error running crontab -l"
    );

    ssh_err!(
        channel.exec(&format!("echo \"@reboot ~/{}\" >> mycrontab", &filename)),
        "Error running echo"
    );

    ssh_err!(
        channel.exec("crontab mycrontab"),
        "Error installing new crontab"
    );

    ssh_err!(channel.exec("rm mycrontab"), "Error removing temp crontab");

    channel.wait_close();
    println!("{}", channel.exit_status().unwrap());
    Ok(())
}

fn upload_file(filename: &str, sess: &Session) -> Result<(), Error> {
    // Read the compiled script into memory
    let mut bin_file = match std::fs::File::open(&filename) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::IOError,
                format!("Unable to open binary file {}", e),
            ))
        }
    };

    let mut bin = vec![];

    match bin_file.read_to_end(&mut bin) {
        Ok(_) => {}
        Err(e) => {
            return Err(Error::new(
                ErrorKind::IOError,
                format!("Could not read binary file, {}", e),
            ))
        }
    };

    // Upload the binary file
    let mut remote_file = ssh_err!(
        sess.scp_send(Path::new(&filename), 0o644, bin.len() as u64 * 8, None),
        "Failed to create file on server"
    );

    ssh_err!(remote_file.write_all(&bin), "Unable to write binary file");
    // Close the channel and wait for the whole content to be tranferred
    ssh_err!(remote_file.send_eof(), "Unable to write binary file");
    ssh_err!(remote_file.wait_eof(), "Error sending to remote");
    ssh_err!(remote_file.close(), "Error closing remote file");
    ssh_err!(remote_file.wait_close(), "Error closing remote file");

    Ok(())
}
