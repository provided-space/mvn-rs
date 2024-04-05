> Simple maven registry written in Rust

# mvn-rs

This web application is capable of handling requests coming from Maven clients. A basic functionality is given for deploying new versions (for authorized clients with write permissions) and reading files.

If an artifact is marked as non-public, the client has to be authorized with read permissions for this artifact.

## Setup

For an up-to-date binary for your production, either build it from source or get it from the Releases.
The `.env` file is used for configurations and the settings work out of the box for a development environment.<br>
Database migrations are built into the library and will be executed before the webserver gets started.

Run the following command in your terminal to start your development environment. This will start the adminer and mysql docker services as well as the application itself.
```bash
docker-compose up -d && cargo run
```

Users, AccessTokens, Groups, Artifacts and Permissions have to be configured manually in the database.

## Authentication

When configuring your repository in your maven project, you have to specify the credentials. (Reading from public artifacts is possible without authentication.)<br>
Credentials in the access_token table are stored as Argon2 hashes.

The username matches the name in your user table. The password is a JWT, pointing to the AccessToken in your database.<br>
Your JWT payload could be something like this:
```json
{
  "id": 1,
  "credentials": "plaintext password"
}
```

## Example cURL request
```curl
curl --location 'http://localhost/com/organisation/library/1.0.0-ALPHA/library-1.0.0-ALPHA.pom' \
--header 'Authorization: Basic ZGV2ZWxvcG1lbnQ6ZXlKaGJHY2lPaUpJVXpJMU5pSXNJblI1Y0NJNklrcFhWQ0o5LmV5SnBaQ0k2TVN3aVkzSmxaR1Z1ZEdsaGJITWlPaUp3YkdGcGJuUmxlSFFnY0dGemMzZHZjbVFpZlEucTZ0YWZtSGcxdG9fT0hOZzl6NWJudDR3dGRHdE1IMTYtaGU2cE1vWGtwaw=='
```