use anyhow::Result;
use std::process::Command;

fn main() -> Result<()> {
  let output = Command::new("git")
    .args(&["rev-parse", "--short", "HEAD"])
    .output()?;
  let git_hash = String::from_utf8(output.stdout)?;
  println!("cargo:rustc-env=GIT_SHORT_HASH={git_hash}");

  Ok(())
}
