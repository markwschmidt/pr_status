use error_chain::error_chain;
use serde::{Deserialize, Serialize};
use std::fs;
use reqwest::{header, blocking::Client};
use log::{info, error};
use std::process::Command;



#[derive(Deserialize)]
struct Config {
    github_username: String,
    access_token: String,
    repo: String
}

error_chain! {
    foreign_links {
        Io(std::io::Error);
        HttpRequest(reqwest::Error);
        JsonError(serde_json::error::Error);
        HeaderError(header::InvalidHeaderValue);
    }
}
#[derive(Deserialize, Debug)]
struct User {
    login: String
}
#[derive(Deserialize, Debug)]
struct Head{
    sha: String
}
#[derive(Deserialize, Debug)]
struct ApiPullRequest {
    url: String,
    html_url: String,
    number: i32,
    state: String,
    user: User,
    statuses_url: String,
    head: Head
}

#[derive(Deserialize, Debug)]
struct ApiPullRequestStatus {
    state: String,
    description: String,
    context: String,
    target_url: String, 
}
#[derive(Deserialize, Serialize, Debug, Eq, PartialEq)]
enum GitlabStatus{
    Pending,
    Running,
    Success,
    Failure,
    Invalid
}

#[derive(Deserialize, Serialize, Debug)]
struct PullRequestStatus{
    number: i32,
    state: GitlabStatus,
    github_url: String,
    pipeline_url: String
}

fn is_done(state: &GitlabStatus) -> bool {
    match state {
        GitlabStatus::Success | GitlabStatus::Failure => true,
        _ => false
    }
}

fn fetch_my_prs(config: &Config, client: &Client) -> Result<Vec<ApiPullRequest>> {
    Ok(client.get(format!("https://api.github.com/repos/{}/pulls", config.repo))
        .send()?
        .json::<Vec<ApiPullRequest>>()?
        .into_iter()
        .filter(|pull| pull.user.login == config.github_username)
        .collect())
}

fn fetch_gitlab_statuses(pulls: Vec<ApiPullRequest>, client: &Client) -> Result<Vec<PullRequestStatus>> {
    let mut new_prs: Vec<PullRequestStatus> = vec![];
    for pull in pulls {
        let statuses: Vec<ApiPullRequestStatus> = client.get(pull.statuses_url).send()?.json()?;
        for status in statuses {
            if status.context == "ci/gitlab/gitlab.com" {
                info!("{:?}", &status);
                let state = match status.state.as_str() {
                    "success" => GitlabStatus::Success,
                    "failure" => GitlabStatus::Failure,
                    "pending" => GitlabStatus::Pending,
                    "running" => GitlabStatus::Running,
                    _ => { error!("Could not parse gitlab status"); GitlabStatus::Invalid}
                };
                new_prs.push(PullRequestStatus { number: pull.number, state: state, pipeline_url: status.target_url, github_url: pull.html_url});
                break; // Take only the newest
            }
        }
    }

    Ok(new_prs)

}

fn notify_pr_finish(status: &PullRequestStatus) -> Result<()> {
    let message = format!("PR #{} {} PR Link: {}", status.number, {
        match status.state {
            GitlabStatus::Success => "succeeded!",
            GitlabStatus::Failure=> "failed!",
            _ => "uhhh"
        }
    }, status.github_url);
    Command::new("ntfy")
        .args(&["send", &message]) 
        .output()?;
    Ok(())
}

fn main() -> Result<()> {

    env_logger::Builder::from_env(env_logger::Env::default().default_filter_or("info")).init();

    let config : Config =  serde_json::from_str(&fs::read_to_string("secrets.txt")?)?;

    // Set up auth
    let mut headers = header::HeaderMap::new();
    headers.append(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("Bearer {}", config.access_token))?);
    headers.append(header::USER_AGENT, header::HeaderValue::from_str(&"Mark's Cool PR Checker")?);

    // Setup client
    let client = Client::builder()
        .default_headers(headers)
        .build()?;
    
    // Fetch the PRs
    let pulls = fetch_my_prs(&config, &client)?;
    info!("Got {} pulls for user", pulls.len());

    // Fetch gitlab status for each PR
    let new_prs = fetch_gitlab_statuses(pulls, &client)?;
    info!("PR statuses: {:?}", new_prs);

    // Load known prs from file
    let known_prs : Vec<PullRequestStatus> = serde_json::from_str(&fs::read_to_string("prs.json").unwrap_or("[]".to_string()))?;
    let mut out_prs: Vec<PullRequestStatus> = vec![];

    // Merge new pulls and old pulls. Notify if anything from old pulls is now done
    for new_pr in new_prs {
        for known_pr in &known_prs {
            if new_pr.number == known_pr.number {
                if is_done(&new_pr.state) && !is_done(&known_pr.state) {
                    info!("Notifying!");
                    notify_pr_finish(&new_pr)?;
                }
            }
        }
        out_prs.push(new_pr);
    }
    fs::write("prs.json", serde_json::to_string_pretty(&out_prs)?)?;
    println!("blah");
    Ok(())
}
