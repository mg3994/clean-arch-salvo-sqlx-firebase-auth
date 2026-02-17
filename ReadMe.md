```angular2html
src/
├── main.rs                 # Entry point: initializes container and starts Salvo
├── lib.rs                  # Optional: exposes modules for integration testing
├── core/                   # <--- THE DOMAIN LAYER (Pure Logic)
│   ├── entities/           # Pure data structures (User, Session)
│   ├── repository/         # Traits (Interfaces) for DB (e.g., trait UserRepo)
│   ├── services/           # Traits for external tools (e.g., trait EmailService)
│   └── errors/             # Custom Error enums (using thiserror)
├── application/            # <--- THE USE CASE LAYER (The "What")
│   ├── auth/               # register_user.rs, login_use_case.rs
│   ├── user/               # update_profile.rs
│   └── dtos.rs             # Request/Response structs (Validation logic here)
├── infrastructure/         # <--- THE DATA LAYER (The "How")
│   ├── persistence/        # SQLx implementations of repository traits
│   ├── external/           # Firebase Admin SDK & Argon2 implementations
│   ├── config/             # Figment & Dotenvy loaders
│   └── logging/            # Tracing setup
├── interface/              # <--- THE API LAYER (Delivery)
│   ├── http/               # Salvo specific logic
│   │   ├── controllers/    # Request Handlers (call Application layer)
│   │   ├── middleware/     # JWT, CORS, Logging (Your "Hoops")
│   │   └── router.rs       # Salvo Router tree
│   └── templates/          # Askama rendering logic & Presenters
└── utils/                  # Shared helpers (UUID v7, pagination)
```


```
cargo add anyhow `
    figment --features env,toml `
    jsonwebtoken --features rust_crypto `
    rust-embed `
    salvo --features anyhow,cookie,cors,jwt-auth,oapi,serve-static,rustls,logging,test,quinn `
    serde --features derive `
    thiserror `
    time `
    tokio --features full `
    tracing `
    validator --features derive `
    argon2 `
    cookie `
    dotenvy `
    tracing-appender `
    tracing-subscriber --features std,fmt,env-filter,tracing-log,time,local-time,json `
    sqlx --features runtime-tokio,macros,postgres,uuid,chrono,json `
    askama `
    rand `
    rustls@0.23 --features ring `
    firebase-admin-sdk --features auth,messaging `
    uuid --features serde,v7,std `
    chrono --features serde `
    serde_json
```


#### To Test FCM TOKEN GENERATION
```powershell
"C:\Program Files\Google\Chrome\Application\chrome.exe" --ignore-certificate-errors --unsafely-treat-insecure-origin-as-secure=https://localhost:8008 --user-data-dir=C:\tmp_chrome
```



```
cargo add anyhow rust-embed thiserror time tracing argon2 cookie dotenvy tracing-appender askama rand serde_json; cargo add figment --features env,toml; cargo add jsonwebtoken --features rust_crypto; cargo add salvo --features anyhow,cookie,cors,jwt-auth,oapi,serve-static,rustls,logging,test,quinn; cargo add serde --features derive; cargo add tokio --features full; cargo add validator --features derive; cargo add tracing-subscriber --features std,fmt,env-filter,tracing-log,time,local-time,json; cargo add sqlx --features runtime-tokio,macros,postgres,uuid,chrono,json; cargo add 'rustls@0.23' --features ring; cargo add firebase-admin-sdk --features auth,messaging; cargo add uuid --features serde,v7,std; cargo add chrono --features serde
```