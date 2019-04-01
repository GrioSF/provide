A command-line tool for retrieving variables from AWS Parameter Store.

# Getting Started
TBD

# Setting up AWS Parameter Store
TBD

## Paths
TBD

# Populating your variables
TBD

## Use `provide` to setup
TBD

## Use `bash` to setup
TBD

# AWS Region Resolution

The underlying lib for this tool appears to emulate the process described at https://docs.aws.amazon.com/sdk-for-java/v1/developer-guide/java-dg-region-selection.html with some differences.

In provider, resolution resolves in this order of priority:

1. Specify a region directly, e.g. `--region us-west-1`
2. Use an environment variable, e.g. `AWS_REGION=us-west-1`
3. Define the region in a profile and use this via `--profile my-profile`
4. The AWS instance metadata service

You may use both `--region` and `--profile`. In this case your profile's  credentials are used and the region specified for `--region` is used instead of any region defined in your profile.

# Setting up an AWS Profile

Modify `~/.aws/credentials`
```
[my-profile]
aws_access_key_id = foo
aws_secret_access_key = bar
```

Modify `~/.aws/config`
```
[profile my-profile]
cli_follow_urlparam = false
region = us-west-1
```

# Working with the code

Clone this repo and run all commands from the root directory.

## Install the Rust toolchain
TBD

## Running in development mode

```
cargo run --bin provide -- --profile my-profile /app/env
```

## Building release and running it

```
cargo build --release
./target/release/provide --profile my-profile /app/env
```
