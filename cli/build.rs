use std::process::Command;

fn main() -> Result<(), Box<dyn std::error::Error>> {
    println!("cargo:rustc-env=FOO=bar");
    let git_dir = "../.git";

    println!("cargo:rerun-if-changed=build.rs");
    println!("cargo:rerun-if-changed={git_dir}/index");
    println!("cargo:rerun-if-changed={git_dir}/HEAD");
    println!("cargo:rerun-if-changed={git_dir}/refs");

    let mut command = Command::new("git");
    command.arg("describe");
    command.arg("--always");
    command.arg("--long");
    let revision = get_command_output(command)?;
    println!("cargo:rustc-env=REVISION={revision}");

    let mut command = Command::new("git");
    command.args([
        "--no-pager",
        "log",
        "-1",
        "--pretty=format:%cd",
        "--date=format:%Y-%m-%d %H:%M:%S",
    ]);
    let last_commit_date = get_command_output(command)?;
    println!("cargo:rustc-env=LAST_COMMIT_DATE={last_commit_date}");

    Ok(())
}

fn get_command_output(mut command: Command) -> Result<String, Box<dyn std::error::Error>> {
    let output = command.output()?;
    let result = String::from_utf8(output.stdout)?;
    Ok(result)
}
