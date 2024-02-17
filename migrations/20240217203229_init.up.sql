CREATE TABLE `user` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `name` varchar(255) NOT NULL,
    PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `access_token` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `user_id` int unsigned NOT NULL,
    `credentials` varchar(255) NOT NULL,
    PRIMARY KEY (`id`),
    KEY `user_id` (`user_id`),
    CONSTRAINT `access_token_ibfk_1` FOREIGN KEY (`user_id`) REFERENCES `user` (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `maven_group` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `name` varchar(255) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `name` (`name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `maven_artifact` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `group_id` int unsigned NOT NULL,
    `name` varchar(255) NOT NULL,
    `public` tinyint NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `group_id_name` (`group_id`,`name`),
    KEY `group_id` (`group_id`),
    CONSTRAINT `maven_artifact_ibfk_1` FOREIGN KEY (`group_id`) REFERENCES `maven_group` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `maven_version` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `artifact_id` int unsigned NOT NULL,
    `version` varchar(16) NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `artifact_id_version` (`artifact_id`,`version`),
    KEY `artifact_id` (`artifact_id`),
    CONSTRAINT `maven_version_ibfk_1` FOREIGN KEY (`artifact_id`) REFERENCES `maven_artifact` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;

CREATE TABLE `maven_file` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `version_id` int unsigned DEFAULT NULL,
    `name` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
    `uri` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
    `path` varchar(255) CHARACTER SET utf8mb4 COLLATE utf8mb4_general_ci NOT NULL,
    PRIMARY KEY (`id`),
    UNIQUE KEY `uri` (`uri`),
    UNIQUE KEY `path` (`path`),
    KEY `version_id` (`version_id`),
    CONSTRAINT `maven_file_ibfk_1` FOREIGN KEY (`version_id`) REFERENCES `maven_version` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
