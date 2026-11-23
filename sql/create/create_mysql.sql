-- RappelzCore : création des bases et de l'utilisateur, comme data/sql/create/create_mysql.sql
-- d'AzerothCore. À exécuter en root, puis importer sql/base/<db>/*.sql et sql/custom/<db>/*.sql.
CREATE USER IF NOT EXISTS 'rcore'@'localhost' IDENTIFIED BY 'rcore' WITH MAX_QUERIES_PER_HOUR 0 MAX_CONNECTIONS_PER_HOUR 0 MAX_UPDATES_PER_HOUR 0;
CREATE USER IF NOT EXISTS 'rcore'@'127.0.0.1' IDENTIFIED BY 'rcore';

CREATE DATABASE IF NOT EXISTS `rcore_world` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE IF NOT EXISTS `rcore_characters` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_unicode_ci;
CREATE DATABASE IF NOT EXISTS `rcore_auth` DEFAULT CHARACTER SET UTF8MB4 COLLATE utf8mb4_unicode_ci;

GRANT ALL PRIVILEGES ON `rcore_world`.* TO 'rcore'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON `rcore_characters`.* TO 'rcore'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON `rcore_auth`.* TO 'rcore'@'localhost' WITH GRANT OPTION;
GRANT ALL PRIVILEGES ON `rcore_world`.* TO 'rcore'@'127.0.0.1';
GRANT ALL PRIVILEGES ON `rcore_characters`.* TO 'rcore'@'127.0.0.1';
GRANT ALL PRIVILEGES ON `rcore_auth`.* TO 'rcore'@'127.0.0.1';
