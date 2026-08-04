use serde::Deserialize;

// ---- Profil user: commits, followers, repos, bahasa ----

#[derive(Deserialize, Debug)]
pub struct GraphQLResponse {
    pub data: DataWrapper,
}

#[derive(Deserialize, Debug)]
pub struct DataWrapper {
    pub user: UserData,
}

#[derive(Deserialize, Debug)]
pub struct UserData {
    #[serde(rename = "contributionsCollection")]
    pub contributions_collection: ContributionsCollection,
    pub followers: Followers,
    pub repositories: Repositories,
}

#[derive(Deserialize, Debug)]
pub struct ContributionsCollection {
    #[serde(rename = "contributionCalendar")]
    pub contribution_calendar: ContributionCalendar,
    #[serde(rename = "totalRepositoriesWithContributedCommits")]
    pub total_repositories_with_contributed_commits: u32,
}

#[derive(Deserialize, Debug)]
pub struct Followers {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
}

#[derive(Deserialize, Debug)]
pub struct ContributionCalendar {
    #[serde(rename = "totalContributions")]
    pub total_contributions: u32,
}

#[derive(Deserialize, Debug)]
pub struct Repositories {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
    pub nodes: Vec<RepoNode>,
}

#[derive(Deserialize, Debug)]
pub struct Stargazers {
    #[serde(rename = "totalCount")]
    pub total_count: u32,
}

#[derive(Deserialize, Debug)]
pub struct RepoNode {
    pub name: String,
    pub stargazers: Stargazers,
    pub languages: LanguageConnection,
}

#[derive(Deserialize, Debug)]
pub struct LanguageConnection {
    pub edges: Vec<LanguageEdge>,
}

#[derive(Deserialize, Debug)]
pub struct LanguageEdge {
    pub size: u64,
    pub node: LanguageNode,
}

#[derive(Deserialize, Debug)]
pub struct LanguageNode {
    pub name: String,
}

// ---- LOC per-repo (paging commit history) ----

#[derive(Deserialize, Debug)]
pub struct RepoLocResponse {
    pub data: RepoLocData,
}

#[derive(Deserialize, Debug)]
pub struct RepoLocData {
    pub repository: Option<RepositoryHistory>,
}

#[derive(Deserialize, Debug)]
pub struct RepositoryHistory {
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: Option<DefaultBranchRef>,
}

#[derive(Deserialize, Debug)]
pub struct DefaultBranchRef {
    pub target: Option<CommitTarget>,
}

#[derive(Deserialize, Debug)]
pub struct CommitTarget {
    pub history: CommitHistory,
}

#[derive(Deserialize, Debug)]
pub struct CommitHistory {
    #[serde(rename = "pageInfo")]
    pub page_info: PageInfo,
    pub edges: Vec<CommitEdge>,
}

#[derive(Deserialize, Debug)]
pub struct PageInfo {
    #[serde(rename = "hasNextPage")]
    pub has_next_page: bool,
    #[serde(rename = "endCursor")]
    pub end_cursor: Option<String>,
}

#[derive(Deserialize, Debug)]
pub struct CommitEdge {
    pub node: CommitNode,
}

#[derive(Deserialize, Debug)]
pub struct CommitNode {
    pub additions: u64,
    pub deletions: u64,
    pub author: CommitAuthor,
}

#[derive(Deserialize, Debug)]
pub struct CommitAuthor {
    pub user: Option<CommitAuthorUser>, // bisa null kalau akun sudah dihapus, dsb
}

#[derive(Deserialize, Debug)]
pub struct CommitAuthorUser {
    pub login: String,
}

// ---- total commit count per-repo (buat cek cache basi/tidaknya) ----

#[derive(Deserialize, Debug)]
pub struct RepoCountResponse {
    pub data: RepoCountData,
}

#[derive(Deserialize, Debug)]
pub struct RepoCountData {
    pub repository: Option<RepoCountRepository>,
}

#[derive(Deserialize, Debug)]
pub struct RepoCountRepository {
    #[serde(rename = "defaultBranchRef")]
    pub default_branch_ref: Option<RepoCountBranch>,
}

#[derive(Deserialize, Debug)]
pub struct RepoCountBranch {
    pub target: Option<RepoCountTarget>,
}

#[derive(Deserialize, Debug)]
pub struct RepoCountTarget {
    pub history: RepoCountHistory,
}

#[derive(Deserialize, Debug)]
pub struct RepoCountHistory {
    #[serde(rename = "totalCount")]
    pub total_count: u64,
}
