// Copyright 2023 Salesforce, Inc. All rights reserved.
mod generated;

use std::collections::HashMap;

use anyhow::{anyhow, Result};
use pdk::hl::*;
use pdk::logger;
use crate::generated::config::Config;

async fn request_filter(request_state: RequestState, config: &Config) {
    let headers_state = request_state.into_headers_state().await;
    let headers_handler = headers_state.handler();

    header_log(&headers_handler.headers(), config);
}

async fn response_filter(response_state: ResponseState, config: &Config) {
    let headers_state = response_state.into_headers_state().await;
    let headers_handler = headers_state.handler();

    header_log(&headers_handler.headers(), config);
}

fn header_log(headers: &[(String, String)], config: &Config) {
    if headers.is_empty() {
        logger::info!("No headers to log.");
        return;
    }

    let masked_text = create_masked_text(headers, config);

    logger::info!("{masked_text}");     
}

fn create_masked_text(headers: &[(String, String)], config: &Config) -> String {
    let mut headers_map = HashMap::new();
    
    for (header_name,  header_value) in headers {
        if config.header.contains(&header_name) {
            headers_map.insert(header_name, "*****");
        } else {
            headers_map.insert(header_name, header_value);
        }
    }

    return serde_json::to_string(&headers_map).unwrap();
}

#[entrypoint]
async fn configure(launcher: Launcher, Configuration(bytes): Configuration) -> Result<()> {
    let config: Config = serde_json::from_slice(&bytes).map_err(|err| {
        anyhow!(
            "Failed to parse configuration '{}'. Cause: {}",
            String::from_utf8_lossy(&bytes),
            err
        )
    })?;
    let filter = 
        on_request(|rs| request_filter(rs, &config)).
        on_response(|rs| response_filter(rs, &config));
    
    launcher.launch(filter).await?;
    Ok(())
}


#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_create_masked_text_with_some_headers_masked() {
        let config = Config {
            header: vec![String::from("X-Secret")],
        };

        let headers = vec![
            (String::from("X-Secret"), String::from("secret_value")),
            (String::from("X-Public"), String::from("public_value")),
        ];

        let result = create_masked_text(&headers, &config);

        // X-Secret should be masked, X-Public should be visible:
        assert_eq!(result, "{\"X-Secret\":\"*****\",\"X-Public\":\"public_value\"}");
    }
}
