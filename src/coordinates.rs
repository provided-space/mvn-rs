#[derive(Clone)]
pub struct File {
    pub group: String,
    pub artifact: String,
    pub version: String,
    pub file: String,
}

impl File {
    pub fn to_version(self) -> Version {
        return Version { group: self.group, artifact: self.artifact, version: self.version };
    }
}

#[derive(Clone)]
pub struct Version {
    pub group: String,
    pub artifact: String,
    pub version: String,
}

impl Version {
    pub fn to_artifact(self) -> Artifact {
        return Artifact { group: self.group, artifact: self.artifact };
    }
}

#[derive(Clone)]
pub struct Artifact {
    pub group: String,
    pub artifact: String,
}

impl Artifact {
    pub fn to_group(self) -> Group {
        return Group { group: self.group };
    }
}

#[derive(Clone)]
pub struct Group {
    pub group: String,
}

pub struct Parser {}

impl Parser {
    pub fn parse_to_file(resource_identifier: &str) -> Result<File, String> {
        let parts = Parser::split(resource_identifier, 4)?;

        let file = parts[parts.len() - 1].to_string();
        let version = parts[parts.len() - 2].to_string();
        let artifact = parts[parts.len() - 3].to_string();
        let group = parts[0..parts.len() - 3].join(".");

        return Ok(File { group, artifact, version, file });
    }

    pub fn parse_to_version(resource_identifier: &str) -> Result<Version, String> {
        let parts = Parser::split(resource_identifier, 3)?;

        let version = parts[parts.len() - 1].to_string();
        let artifact = parts[parts.len() - 2].to_string();
        let group = parts[0..parts.len() - 2].join(".");

        return Ok(Version { group, artifact, version });
    }

    pub fn parse_to_artifact(resource_identifier: &str) -> Result<Artifact, String> {
        let parts = Parser::split(resource_identifier, 2)?;

        let artifact = parts[parts.len() - 1].to_string();
        let group = parts[0..parts.len() - 1].join(".");

        return Ok(Artifact { group, artifact });
    }

    pub fn parse_to_group(resource_identifier: &str) -> Result<Group, String> {
        let parts = Parser::split(resource_identifier, 1)?;

        let group = parts[0..parts.len()].join(".");

        return Ok(Group { group });
    }

    fn split(resource_identifier: &str, minimum_length: usize) -> Result<Vec<&str>, String> {
        let parts: Vec<&str> = resource_identifier.trim_start_matches('/').split('/').collect();
        if parts.len() >= minimum_length {
            return Ok(parts);
        }
        return Err("Identifier is too short.".to_string());
    }
}
