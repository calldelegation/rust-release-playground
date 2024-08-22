use octocrab::Octocrab;
use octocrab::params::State; // Import the State enum
use regex::Regex;
use octocrab::commits::PullRequestTarget;

#[derive(Debug)]
pub struct ChangelogInfo {
    pub is_breaking: bool,
    pub pr_type: String,
    pub bullet_point: String,
    pub migration_note: String,
    pub release_notes: String,
}

pub fn capitalize(s: &str) -> String {
    let mut c = s.chars();
    match c.next() {
        None => String::new(),
        Some(f) => f.to_uppercase().collect::<String>() + c.as_str(),
    }
}

pub async fn get_changelog_info(
    octocrab: &Octocrab,
    commit_sha: &str,
) -> Result<ChangelogInfo, Box<dyn std::error::Error>> {
    let pr_info = octocrab
        .repos("calldelegation", "rust-release-playground")
        .list_pulls(commit_sha.to_string())
        .send()
        .await?;
    
    if pr_info.items.is_empty() {
        return Err("No PR found for this commit SHA".into());
    }
    
    let pr = &pr_info.items[0];

    let pr_type = pr
        .title
        .as_ref()
        .map_or("misc", |title| title.split(':').next().unwrap_or("misc"))
        .to_string();
    let is_breaking = pr.title.as_ref().map_or(false, |title| title.contains("!"));

    let title_description = pr
        .title
        .as_ref()
        .map_or("", |title| title.split(':').nth(1).unwrap_or(""))
        .trim()
        .to_string();
    let bullet_point = format!(
        "- {} - {}, by {}",
        pr.html_url.as_ref().map_or("", |url| url.as_str()),
        capitalize(&title_description),
        pr.user.as_ref().map_or("", |user| &user.login)
    );

    let breaking_changes_regex = Regex::new(r"# Breaking Changes([\s\S]+?)#").unwrap();
    let breaking_changes = breaking_changes_regex
        .captures(&pr.body.as_ref().unwrap_or(&String::new()))
        .and_then(|cap| cap.get(1))
        .map_or_else(|| String::new(), |m| m.as_str().trim().to_string());

    let release_notes_regex = Regex::new(r"In this release, we:([\s\S]+?)#").unwrap();
    let release_notes = release_notes_regex
        .captures(&pr.body.as_ref().unwrap_or(&String::new()))
        .and_then(|cap| cap.get(1))
        .map_or_else(|| String::new(), |m| m.as_str().trim().to_string());

    let migration_note = format!(
        "### [{} - {}]({})\n\n{}",
        pr.number,
        capitalize(&title_description),
        pr.html_url.as_ref().map_or("", |url| url.as_str()),
        breaking_changes
    );

    Ok(ChangelogInfo {
        is_breaking,
        pr_type,
        bullet_point,
        migration_note,
        release_notes,
    })
}

pub async fn get_changelogs(
    octocrab: &Octocrab,
    owner: &str,
    repo: &str,
    base: &str,
    head: &str,
) -> Result<Vec<ChangelogInfo>, Box<dyn std::error::Error>> {
    println!("BASE: {:?}", base);
    println!("HEAD: {:?}", head);
    println!("REPO: {:?}", repo);
    println!("OWNER: {:?}", owner);
    let comparison = octocrab.commits(owner, repo).compare(base, head).send().await?;
    
    println!("Comparison status: {:?}", comparison.status);
    
    // Add debug info
    println!("Found {} commits in the comparison", comparison.commits.len());

    let mut changelogs = Vec::new();

    for commit in comparison.commits {
        println!("Commit SHA: {}", commit.sha);
        let info = get_changelog_info(&octocrab, &commit.sha).await?;
        println!("Changelog Info: {:?}", info);
        changelogs.push(info);
    }

    changelogs.sort_by(|a, b| a.pr_type.cmp(&b.pr_type));

    Ok(changelogs)
}

pub fn generate_changelog(changelogs: Vec<ChangelogInfo>) -> String {
    let mut content = String::new();

    let release_notes: String = changelogs
        .iter()
        .filter(|c| !c.release_notes.is_empty())
        .map(|c| c.release_notes.clone())
        .collect::<Vec<_>>()
        .join("\n");

    let breaking: String = changelogs
        .iter()
        .filter(|c| c.is_breaking)
        .map(|c| c.bullet_point.clone())
        .collect::<Vec<_>>()
        .join("\n");

    let non_breaking: String = changelogs
        .iter()
        .filter(|c| !c.is_breaking)
        .map(|c| c.bullet_point.clone())
        .collect::<Vec<_>>()
        .join("\n");

    let migration_notes: String = changelogs
        .iter()
        .filter(|c| c.is_breaking)
        .map(|c| c.migration_note.clone())
        .collect::<Vec<_>>()
        .join("\n\n");

    if !release_notes.is_empty() {
        content.push_str(&format!(
            "# Summary\n\nIn this release, we:\n{}\n\n",
            release_notes
        ));
    }
    if !breaking.is_empty() {
        content.push_str(&format!("# Breaking\n\n{}\n\n", breaking));
    }
    if !non_breaking.is_empty() {
        content.push_str(&format!("{}\n\n", non_breaking));
    }
    if !migration_notes.is_empty() {
        content.push_str(&format!("# Migration Notes\n\n{}\n\n", migration_notes));
    }

    content.trim().to_string()
}
