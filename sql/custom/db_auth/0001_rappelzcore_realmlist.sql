-- RappelzCore : royaume par défaut.
-- La structure de `realmlist` est celle d'AzerothCore ; seules les valeurs changent.
-- `port` = WorldServerPort du worldserver (port joueur Rappelz), `gamebuild` = version client.
UPDATE `realmlist`
   SET `name` = 'RappelzCore', `address` = '127.0.0.1', `localAddress` = '127.0.0.1', `port` = 4514,
       `icon` = 0, `flag` = 0, `timezone` = 1, `allowedSecurityLevel` = 0, `gamebuild` = 10360
 WHERE `id` = 1;
