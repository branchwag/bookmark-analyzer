use reqwest;
use serde::{Deserialize, Serialize};

#[derive(Debug, Serialize)]
struct OllamaRequest {
    model: String,
    prompt: String,
    stream: bool,
}

#[derive(Debug, Deserialize)]
struct OllamaResponse {
    response: String,
}

pub async fn analyze_bookmarks(
    bookmarks: &[crate::browser::Bookmark],
) -> Result<String, Box<dyn std::error::Error>> {
    // Build the prompt
    let bookmark_list: Vec<String> = bookmarks
        .iter()
        .map(|b| format!("- {}: {}", b.name, b.url))
        .collect();

    let prompt = format!(
        "You are an insightful analyst. Based on someone's browser bookmarks, provide a thoughtful reflection about their interests, habits, and personality. Be creative and engaging.\n\nBookmarks:\n{}\n\nProvide a 2-3 paragraph reflection:",
        bookmark_list.join("\n")
    );

    let client = reqwest::Client::new();
    let request = OllamaRequest {
        model: "llama3.2".to_string(),
        prompt,
        stream: false,
    };

    let response = client
        .post("http://localhost:11434/api/generate")
        .json(&request)
        .send()
        .await?;

    let ollama_response: OllamaResponse = response.json().await?;

    Ok(ollama_response.response)
}
