pub async fn auth_middleware(request: &mut Request) {
	if let Some(session_token) = extract_session_token(request.cookie_header) {
		//Find token in Database
	}
} 

pub fn extract_session_token(cookie_header: &str) -> Option<String> {
    for cookie_pair in cookie_header.split(';') {
        let trimmed_pair = cookie_pair.trim(); 

        if let Some((key, value)) = trimmed_pair.split_once('=') {
            if key == "session_id" {
                return Some(value.to_string());
            }
        }
    }
    
    None
}