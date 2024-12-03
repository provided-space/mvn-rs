use dotenv::dotenv;

pub fn parse() -> ApplicationConfig {
    if dotenv().is_err() {
        println!("Couldn't find .env file. Loading config from environment.");
    }

    return ApplicationConfig {
        database: DatabaseConfig {
            url: env_var("DATABASE_URL"),
        },
        webserver: WebserverConfig {
            host: env_var("WEBSERVER_HOST"),
            port: env_var("WEBSERVER_PORT").parse::<u16>().expect("Could not cast WEBSERVER_PORT to port."),
        },
        authentication: AuthenticationConfig {
            secret: env_var("AUTHENTICATION_SECRET"),
        },
        file_storage: env_var("FILE_STORAGE"),
    };
}

fn env_var(var: &str) -> String {
    return std::env::var(var).expect(&*format!("Missing {var}."));
}

#[derive(Clone)]
pub struct ApplicationConfig {
    pub database: DatabaseConfig,
    pub webserver: WebserverConfig,
    pub authentication: AuthenticationConfig,
    pub file_storage: String,
}

#[derive(Clone)]
pub struct WebserverConfig {
    pub host: String,
    pub port: u16,
}

impl WebserverConfig {
    pub fn to_address(&self) -> (&str, u16) {
        return (self.host.as_str(), self.port);
    }
}

#[derive(Clone)]
pub struct DatabaseConfig {
    pub url: String,
}

#[derive(Clone)]
pub struct AuthenticationConfig {
    pub secret: String,
}
