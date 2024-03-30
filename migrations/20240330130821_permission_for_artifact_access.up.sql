CREATE TABLE `maven_permission` (
    `id` int unsigned NOT NULL AUTO_INCREMENT,
    `access_token_id` int unsigned NOT NULL,
    `entity_type` varchar(255) COLLATE utf8mb4_general_ci NOT NULL,
    `artifact_id` int unsigned DEFAULT NULL,
    `read` tinyint NOT NULL,
    `write` tinyint NOT NULL,
    PRIMARY KEY (`id`),
    KEY `access_token_id` (`access_token_id`),
    KEY `artifact_id` (`artifact_id`),
    CONSTRAINT `maven_permission_ibfk_1` FOREIGN KEY (`access_token_id`) REFERENCES `access_token` (`id`) ON DELETE RESTRICT,
    CONSTRAINT `maven_permission_ibfk_2` FOREIGN KEY (`artifact_id`) REFERENCES `maven_artifact` (`id`) ON DELETE RESTRICT
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_general_ci;
