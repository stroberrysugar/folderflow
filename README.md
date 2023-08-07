# Folderflow

## Overview
Let's you upload folders, run scripts on them, and download them back as ZIP files.

## Features
1. Upload folders
2. Download folders as ZIP files
3. Get list of current folders (and displays the time of creation)
4. Get list of scripts
5. Execute a script in a specific folder (and displays stdout/stderr)

NOTE: You will only be able to define a script in the config file as described below.

## Demo

In this demo, the script file just contains `ls -lh` to list all the files in the current directory.

https://github.com/stroberrysugar/folderflow/assets/45601318/262c7754-ec43-4b34-b56f-6ad3fe97d02c

## Getting started

1. Install Rust: `curl https://sh.rustup.rs -sSf | sh -s -- -y`
2. Source Rust's ENV file: `source "$HOME/.cargo/env"`
3. Clone this repo: `git clone https://github.com/stroberrysugar/folderflow.git`
4. Navigate to the repo and do `cargo run --release`
5. Browse `http://localhost:8082`

You would typically want to run this using a systemd unit or a screen session.

## Configuration

The config.toml file is fairly straightforward. Some important points to keep in mind:

1. You will need to set `root_folder_directory`. This is the directory where all your folders get uploaded to
2. `temp_zip_directory` is where the temporary generated ZIP files will be in when downloading a folder
3. All scripts must have unique IDs (from 0 to 65535)
4. Ensuring that permissions are set up correctly in `root_folder_directory` and other directories is on you

```toml
listen_address = "0.0.0.0:8082"
root_folder_directory = "/projects/folderflow/uploaded_folders"
temp_zip_directory = "/tmp"
max_upload_size_in_bytes = 4294967296

# just add scripts in this pattern

[[script_config]]
id = 0
friendly_name = "script 1"
path_to_script = "/home/user/testscript.sh"

[[script_config]]
id = 1
friendly_name = "script 2"
path_to_script = "/home/user/testscript2.sh"
```
