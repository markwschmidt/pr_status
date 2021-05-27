use error_chain::error_chain;
use std::io::Read;
use serde::{Deserialize, Serialize};
use std::fs;
use reqwest::{header, blocking::Client};


#[derive(Deserialize)]
struct Config {
    github_username: String,
    access_token: String
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

#[derive(Deserialize, Serialize, Debug)]
struct PullRequestStatus{
    number: i32,
    state: String,
    pipeline_url: String
}

fn is_done(state: &str) -> bool {
    match state {
        "success" | "failure" => true,
        _ => false
    }
}

fn main() -> Result<()> {

    let config : Config =  serde_json::from_str(&fs::read_to_string("secrets.txt")?)?;

    // Set up auth
    let mut headers = header::HeaderMap::new();
    headers.append(header::AUTHORIZATION, header::HeaderValue::from_str(&format!("Bearer {}", config.access_token))?);
    headers.append(header::USER_AGENT, header::HeaderValue::from_str(&"Mark's Cool PR Checker")?);

    // Fetch the PRs
    let client = Client::builder()
        .default_headers(headers)
        .build()?;
    let all_pulls : Vec<ApiPullRequest> = client.get("https://api.github.com/repos/Xeograph/xgsrc/pulls").send()?.json()?;
    let my_pulls :Vec<ApiPullRequest> =all_pulls.into_iter().filter(|pull| pull.user.login == config.github_username).collect();

    println!("Got {} pulls for user", my_pulls.len());


    // Fetch status for each old pull
    let mut new_prs: Vec<PullRequestStatus> = vec![];
    for my_pull in my_pulls {
        let statuses: Vec<ApiPullRequestStatus> = client.get(my_pull.statuses_url).send()?.json()?;
        for status in statuses {
            if status.context == "ci/gitlab/gitlab.com" {
                println!("{:?}", status);
                new_prs.push(PullRequestStatus { number: my_pull.number, state: status.state, pipeline_url: status.target_url});
                break; // Take only the newest
            }
        }
    }

    println!("PR statuses: {:?}", new_prs);

    let mut known_prs : Vec<PullRequestStatus> = serde_json::from_str(&fs::read_to_string("prs.json").unwrap_or("[]".to_string()))?;
    let mut out_prs: Vec<PullRequestStatus> = vec![];

    // Merge new pulls and old pulls. Notify if anything from old pulls is now done
    for new_pr in new_prs {
        for known_pr in &known_prs {
            if new_pr.number == known_pr.number {
                // TODO figure out DONE
                if is_done(&new_pr.state) && !is_done(&known_pr.state) {
                    // TODO notify
                    println!("Notifying!");
                }
            }
        }
        out_prs.push(PullRequestStatus { number: new_pr.number, state: new_pr.state, pipeline_url: new_pr.pipeline_url});
    }
    fs::write("prs.json", serde_json::to_string_pretty(&out_prs)?)?;
    Ok(())
}
