# PR Status

This is a simple utility that can be used by Ocient employees to automatically check the status of their PRs

There's quite a bit of setup, so please follow these instructions closely and message me if you need help!
Open to PRs and any other improvements for this (or just bake it into CI so I don't have to do hacky shit like this)

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

I use the Pushover integration to get notifications sent to my phone/desktop. There's tons of other options for sending commands--you can use Slack, Telegram, or Pushbullet. You can even get local desktop notifications on Mac/Linux if you run this on your host machine rather than dev container.

I also use `ntfy` to get notifications when long builds/test runs finish--just use `ntfy done bazel build ...` to get a notification when it finishes. 

You can test it with `ntfy send "this is a test"`

7. Run `cargo run`

This will build and run the binary. You should see a "prs.json" file created in the root directory with info about all of your active PRs. 

This is a good time to test the notification features. If you have a finished PR waiting in CI, change its status in `prs.json` to "Pending", then run `cargo run` again. It should attempt to notify you.

8. Update cron to run every 5 minutes

Run `crontab -e` and add these lines:


Add `cargo` and `ntfy` to your path in crontab. This is where `cargo` and `ntfy` were located for me, obviously change this to what is correct for you
```
PATH=/home/<local_username>/.cargo/bin:/home/<local_username>/.local/bin/:$PATH 
```

Run the command every five minutes
```
*/5 * * * * cd <YOUR/PR_STATUS/DIRECTORY/WITH/SECRETS.TXT> && cargo run > pr_status.log
``

9. Wait for PRs to finish and get notified!