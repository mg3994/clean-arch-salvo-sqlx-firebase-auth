use salvo::jwt_auth::HeaderFinder;

pub fn get_token_finders() -> Vec<Box<dyn salvo::jwt_auth::JwtTokenFinder>> {
    vec![
        Box::new(HeaderFinder::new()),
        // Add more if needed, e.g. CookieFinder
    ]
}

pub fn is_secure_context() -> bool {
    // Logic to check if we are in a secure context (HTTPS)
    // For now, return false to allow HTTP in local dev
    false
}
