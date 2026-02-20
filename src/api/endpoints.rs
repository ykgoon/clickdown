//! ClickUp API endpoint definitions

/// Base URL for ClickUp API v2
pub const BASE_URL: &str = "https://api.clickup.com/api/v2";

/// API endpoint paths
pub struct ApiEndpoints;

impl ApiEndpoints {
    // Workspace/Team endpoints
    pub fn teams() -> String {
        format!("{}/team", BASE_URL)
    }

    // Space endpoints
    pub fn spaces(team_id: &str) -> String {
        format!("{}/team/{}/space", BASE_URL, team_id)
    }

    pub fn space(space_id: &str) -> String {
        format!("{}/space/{}", BASE_URL, space_id)
    }

    // Folder endpoints
    pub fn folders(space_id: &str) -> String {
        format!("{}/space/{}/folder", BASE_URL, space_id)
    }

    pub fn folder(folder_id: &str) -> String {
        format!("{}/folder/{}", BASE_URL, folder_id)
    }

    // List endpoints
    pub fn lists_in_folder(folder_id: &str) -> String {
        format!("{}/folder/{}/list", BASE_URL, folder_id)
    }

    pub fn lists_in_space(space_id: &str) -> String {
        format!("{}/space/{}/list", BASE_URL, space_id)
    }

    pub fn list(list_id: &str) -> String {
        format!("{}/list/{}", BASE_URL, list_id)
    }

    // Task endpoints
    pub fn tasks_in_list(list_id: &str, query: &str) -> String {
        format!("{}/list/{}/task{}", BASE_URL, list_id, query)
    }

    pub fn task(task_id: &str) -> String {
        format!("{}/task/{}", BASE_URL, task_id)
    }

    pub fn tasks_in_team(team_id: &str, query: &str) -> String {
        format!("{}/team/{}/task{}", BASE_URL, team_id, query)
    }

    // Document endpoints
    pub fn docs(query: &str) -> String {
        format!("{}/docs{}", BASE_URL, query)
    }

    pub fn doc_pages(doc_id: &str) -> String {
        format!("{}/doc/{}/pages", BASE_URL, doc_id)
    }

    pub fn page(page_id: &str) -> String {
        format!("{}/page/{}", BASE_URL, page_id)
    }
}
