use std::{
    io::{Read, Write},
    net::TcpStream,
    path::Path,
};

use ssh2::Session;

use crate::error::{Error, ErrorKind};

macro_rules! ssh_err {
    ($stmt:expr,$msg:expr) => {
        match $stmt {
            Ok(a) => a,
            Err(e) => return { Err(Error::new(ErrorKind::SSHError, format!("{} {}", $msg, e))) },
        }
    };
}

pub fn create_session(
    host: String,
    port: usize,
    username: String,
    password: Option<String>,
    key: Option<String>,
) -> Result<Session, Error> {
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

    Ok(sess)
}

pub fn create_cron_job(filename: &str, sess: &Session) -> Result<(), Error> {
    run_command(&format!("chmod +x {}", &filename), &sess)?;
    // Save current crontab into a temp file
    // $ crontab -l > mycrontab
    match run_command("crontab -l > mycrontab", &sess) {
        Ok(_) => {}
        Err(e) => {
            if e.message.ends_with("non-zero exit code") {
                run_command("touch mycrontab", &sess)?;
            } else {
                return Err(e);
            }
        }
    };
    // Add the line to run cf-update on reboot to crontab
    // $ echo "@reboot <FILE>" >> mycrontab
    run_command(
        &format!("echo \"@reboot ~/{}\" >> mycrontab", &filename),
        &sess,
    )?;
    // Install the temp crontab
    // $ crontab mycrontab
    run_command("crontab mycrontab", &sess)?;
    // Remove the temporary crontab
    // $ rm mycrontab
    run_command("rm mycrontab", &sess)?;

    Ok(())
}

fn run_command(command: &str, sess: &Session) -> Result<String, Error> {
    // Create a channel to send the command
    let mut channel = ssh_err!(sess.channel_session(), "Error creating SSH Channel");
    // Run the command
    ssh_err!(
        channel.exec(command),
        format!("Error running command {}", command)
    );
    let mut s = String::new();
    // Get the result of running the command and save into string
    ssh_err!(
        channel.read_to_string(&mut s),
        format!("Error running command {}", command)
    );
    // Close the channel
    ssh_err!(
        channel.wait_close(),
        format!("Error running command, {}", &command)
    );
    // Get the exit code and check that it is 0
    let exit = ssh_err!(
        channel.exit_status(),
        format!("Error running command {}", command)
    );

    if exit != 0 {
        return Err(Error::new(
            ErrorKind::SSHError,
            format!("Command '{}' finished with non-zero exit code", command),
        ));
    }

    // Return the output
    Ok(s)
}

pub fn upload_file(filename: &str, sess: &Session) -> Result<(), Error> {
    // Read the compiled script into memory
    let mut bin_file = match std::fs::File::open(&filename) {
        Ok(x) => x,
        Err(e) => {
            return Err(Error::new(
                ErrorKind::IOError,
                format!("Unable to open binary file {}", e),
            ));
        }
    };

    let mut bin = vec![];

    match bin_file.read_to_end(&mut bin) {
        Ok(_) => {}
        Err(e) => {
            return Err(Error::new(
                ErrorKind::IOError,
                format!("Could not read binary file, {}", e),
            ));
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
