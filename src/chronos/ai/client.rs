use reqwest::Client;
use serde::{Deserialize, Serialize};
use std::env;
use crate::chronos::risk::analyzer::RiskAssessment;
use crate::chronos::state::tracker::FsTarget;

#[derive(Debug, Serialize, Deserialize)]
pub struct AIAnalysis {
    pub intent: String,
    pub explanation: String,
    pub recommendation: String,
}

pub async fn analyze_command(
    command: &str,
    assessment: &RiskAssessment,
    targets: &[FsTarget],
) -> Result<AIAnalysis, Box<dyn std::error::Error>> {
    let api_key = env::var("GEMINI_API_KEY").map_err(|_| "GEMINI_API_KEY not found in .env")?;
    let client = Client::new();

    let url = format!(
        "https://generativelanguage.googleapis.com/v1beta/models/gemini-3.5-flash:generateContent?key={}",
        api_key
    );

    let prompt = format!(
        "You are the AI Semantic Layer for Chronos, a transactional POSIX shell.\n\
        Analyze the following command context:\n\n\
        Command: {}\n\
        Deterministic Risk Level: {:?}\n\
        Targets Affected: {:?}\n\n\
        Return a JSON object with exactly three string keys:\n\
        - 'intent': A one-sentence summary of what the user is trying to accomplish.\n\
        - 'explanation': A brief breakdown of how the command works and what files it impacts.\n\
        - 'recommendation': Advise whether to 'Proceed', 'Proceed with transaction', or 'Block' (e.g. if it's highly destructive like rm -rf /).",
        command, assessment.level, targets
    );

    let body = serde_json::json!({
        "contents": [{
            "parts": [{"text": prompt}]
        }],
        "generationConfig": {
            "responseMimeType": "application/json"
        }
    });

    let res = client.post(&url)
        .json(&body)
        .send()
        .await?;

    let res_json: serde_json::Value = res.json().await?;

    // Check if the API returned an explicit error (like invalid API key)
    if let Some(err) = res_json.get("error") {
        return Err(format!("Google API Error: {}", err).into());
    }

    if let Some(text_response) = res_json["candidates"][0]["content"]["parts"][0]["text"].as_str() {
        // Strip out markdown code fences if Gemini included them
        let mut clean_text = text_response.trim();
        if clean_text.starts_with("```json") {
            clean_text = &clean_text[7..];
        } else if clean_text.starts_with("```") {
            clean_text = &clean_text[3..];
        }
        if clean_text.ends_with("```") {
            clean_text = &clean_text[..clean_text.len()-3];
        }
        clean_text = clean_text.trim();

        // Attempt to parse the cleaned JSON
        match serde_json::from_str(clean_text) {
            Ok(analysis) => Ok(analysis),
            Err(e) => Err(format!("JSON Parse Error: {}. Raw text: {}", e, clean_text).into())
        }
    } else {
        Err(format!("Unexpected API response format: {}", res_json).into())
    }
}
