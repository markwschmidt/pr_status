# PR Status

This is a simple utility that can be used by Ocient employees to automatically check the status of their PRs

## Installation

1. Install libssl-dev

```
sudo apt-get install -y libssl-dev.
```

2. Install Rust/Cargo

```
curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh
```

3. Clone the repo

```
git clone https://github.com/markwschmidt/pr_status
```

4. Create a GitHub OAuth Access Token

On Github.com, go to User Settings from the dropdown in the top right.

Then, navigate to Developer Settings->Personal Access Tokens

Click "Generate new token"

Give it a name like "PR Status Checker" and give it all "repo" privileges

Save this token for your own use

5. Setup a secrets.txt

In the root directory of pr_status, create a file called secrets.txt with the following format:

```
{
   "github_username": "<GITHUB_USERNAME>",
   "access_token": "<ACCESS_TOKEN_FROM_STEP_4>" ,
   "repo": "<REPO_OWNER>/<REPO_NAME>"
}
```

6. Set up ntfy

https://github.com/dschep/ntfy

I use the Pushbullet integration to get notifications sent to my phone/desktop
It's also useful to get notified when a build or test run finishes

7. Run `cargo run`

This will build and run the binary. You should see a "prs.json" file created in the root directory with all 

8. Update cron to run every 5 minutes

Run `crontab -e` and add this line

```

``
