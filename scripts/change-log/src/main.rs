mod get_full_changelog;
mod get_latest_release;

use get_full_changelog::{get_changelogs, generate_changelog};
use get_latest_release::get_latest_release_tag;
use octocrab::Octocrab;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    dotenv::dotenv().ok();
    let github_token = std::env::var("GITHUB_TOKEN").expect("GITHUB_TOKEN is not set in the environment");

    let octocrab = Octocrab::builder().personal_token(github_token).build()?;

    // let latest_release_tag = get_latest_release_tag().await?;

    // let changelogs = get_changelogs(&octocrab, "FuelLabs", "fuels-ts", &latest_release_tag, "master").await?;
    let changelogs = get_changelogs(&octocrab, "FuelLabs", "fuels-ts", "v0.91.0", "v0.92.0").await?;
    let full_changelog = generate_changelog(changelogs);

    println!("{}", full_changelog);

    Ok(())
}
