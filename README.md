A command-line tool for defining and retrieving variables from AWS Parameter Store, so you may centrally manage and provide environment variables for devops purposes.

# Installation
Download the binary for your platform from the release page.

# Usage

Given you have setup variables in AWS Parameter Store with the following root path
`/myapp/staging` and you have an executable `myexecutable`, `provide` will retrieve your variables and provide them to your executable in a sub-process:

```
provide --get -a myapp -t staging ./myexecutable
```

# AWS Region Resolution

The underlying lib for this tool, rusoto, appears to emulate the process described at https://docs.aws.amazon.com/sdk-for-java/v1/developer-guide/java-dg-region-selection.html with some differences.

In `provide`, resolution resolves in this order of priority:

1. Specify a region directly, e.g. `--region us-west-1`
2. Use an environment variable, e.g. `AWS_REGION=us-west-1`
3. Define the region in a profile and use this via `--profile my-profile`
4. The AWS instance metadata service

You may use both `--region` and `--profile`. In this case the region specified for `--region` is used instead of any region defined in your profile and the credentials from the profile are still used.

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
