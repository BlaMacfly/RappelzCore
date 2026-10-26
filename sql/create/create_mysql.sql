-- RappelzCore : création des bases et de l'utilisateur, comme data/sql/create/create_mysql.sql
-- d'AzerothCore. À exécuter en root, puis importer sql/base/<db>/*.sql et sql/custom/<db>/*.sql.
CREATE USER IF NOT EXISTS 'acore'@'localhost' IDENTIFIED BY 'acore' WITH MAX_QUERIES_PER_HOUR 0 MAX_CONNECTIONS_PER_HOUR 0 MAX_UPDATES_PER_HOUR 0;
CREATE USER IF NOT EXISTS 'acore'@'127.0.0.1' IDENTIFIED BY 'acore';

CREATE DATABASE IF NOT EXISTS `acore_world` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE IF NOT EXISTS `acore_characters` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE IF NOT EXISTS `acore_auth` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_unicode_ci;

GRANT ALL PRIVILEGES ON `acore_world`.* TO 'acore'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON `acore_characters`.* TO 'acore'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON `acore_auth`.* TO 'acore'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON `acore_world`.* TO 'acore'@'127.0.0.1';
GRANT ALL PRIVILEGES ON `acore_characters`.* TO 'acore'@'127.0.0.1';
GRANT ALL PRIVILEGES ON `acore_auth`.* TO 'acore'@'127.0.0.1';
