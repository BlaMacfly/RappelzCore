-- MySQL dump 10.13  Distrib 8.4.3, for Win64 (x86_64)
--
-- Host: localhost    Database: acore_world
-- ------------------------------------------------------
-- Server version	8.4.3

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!50503 SET NAMES utf8mb4 */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `npc_text`
--

DROP TABLE IF EXISTS `npc_text`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!50503 SET character_set_client = utf8mb4 */;
CREATE TABLE `npc_text` (
  `ID` int unsigned NOT NULL DEFAULT '0',
  `text0_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text0_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID0` int NOT NULL DEFAULT '0',
  `lang0` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability0` float NOT NULL DEFAULT '0',
  `em0_0` smallint unsigned NOT NULL DEFAULT '0',
  `em0_1` smallint unsigned NOT NULL DEFAULT '0',
  `em0_2` smallint unsigned NOT NULL DEFAULT '0',
  `em0_3` smallint unsigned NOT NULL DEFAULT '0',
  `em0_4` smallint unsigned NOT NULL DEFAULT '0',
  `em0_5` smallint unsigned NOT NULL DEFAULT '0',
  `text1_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text1_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID1` int NOT NULL DEFAULT '0',
  `lang1` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability1` float NOT NULL DEFAULT '0',
  `em1_0` smallint unsigned NOT NULL DEFAULT '0',
  `em1_1` smallint unsigned NOT NULL DEFAULT '0',
  `em1_2` smallint unsigned NOT NULL DEFAULT '0',
  `em1_3` smallint unsigned NOT NULL DEFAULT '0',
  `em1_4` smallint unsigned NOT NULL DEFAULT '0',
  `em1_5` smallint unsigned NOT NULL DEFAULT '0',
  `text2_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text2_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID2` int NOT NULL DEFAULT '0',
  `lang2` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability2` float NOT NULL DEFAULT '0',
  `em2_0` smallint unsigned NOT NULL DEFAULT '0',
  `em2_1` smallint unsigned NOT NULL DEFAULT '0',
  `em2_2` smallint unsigned NOT NULL DEFAULT '0',
  `em2_3` smallint unsigned NOT NULL DEFAULT '0',
  `em2_4` smallint unsigned NOT NULL DEFAULT '0',
  `em2_5` smallint unsigned NOT NULL DEFAULT '0',
  `text3_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text3_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID3` int NOT NULL DEFAULT '0',
  `lang3` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability3` float NOT NULL DEFAULT '0',
  `em3_0` smallint unsigned NOT NULL DEFAULT '0',
  `em3_1` smallint unsigned NOT NULL DEFAULT '0',
  `em3_2` smallint unsigned NOT NULL DEFAULT '0',
  `em3_3` smallint unsigned NOT NULL DEFAULT '0',
  `em3_4` smallint unsigned NOT NULL DEFAULT '0',
  `em3_5` smallint unsigned NOT NULL DEFAULT '0',
  `text4_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text4_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID4` int NOT NULL DEFAULT '0',
  `lang4` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability4` float NOT NULL DEFAULT '0',
  `em4_0` smallint unsigned NOT NULL DEFAULT '0',
  `em4_1` smallint unsigned NOT NULL DEFAULT '0',
  `em4_2` smallint unsigned NOT NULL DEFAULT '0',
  `em4_3` smallint unsigned NOT NULL DEFAULT '0',
  `em4_4` smallint unsigned NOT NULL DEFAULT '0',
  `em4_5` smallint unsigned NOT NULL DEFAULT '0',
  `text5_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text5_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID5` int NOT NULL DEFAULT '0',
  `lang5` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability5` float NOT NULL DEFAULT '0',
  `em5_0` smallint unsigned NOT NULL DEFAULT '0',
  `em5_1` smallint unsigned NOT NULL DEFAULT '0',
  `em5_2` smallint unsigned NOT NULL DEFAULT '0',
  `em5_3` smallint unsigned NOT NULL DEFAULT '0',
  `em5_4` smallint unsigned NOT NULL DEFAULT '0',
  `em5_5` smallint unsigned NOT NULL DEFAULT '0',
  `text6_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text6_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID6` int NOT NULL DEFAULT '0',
  `lang6` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability6` float NOT NULL DEFAULT '0',
  `em6_0` smallint unsigned NOT NULL DEFAULT '0',
  `em6_1` smallint unsigned NOT NULL DEFAULT '0',
  `em6_2` smallint unsigned NOT NULL DEFAULT '0',
  `em6_3` smallint unsigned NOT NULL DEFAULT '0',
  `em6_4` smallint unsigned NOT NULL DEFAULT '0',
  `em6_5` smallint unsigned NOT NULL DEFAULT '0',
  `text7_0` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `text7_1` longtext CHARACTER SET utf8mb4 COLLATE utf8mb4_unicode_ci,
  `BroadcastTextID7` int NOT NULL DEFAULT '0',
  `lang7` tinyint unsigned NOT NULL DEFAULT '0',
  `Probability7` float NOT NULL DEFAULT '0',
  `em7_0` smallint unsigned NOT NULL DEFAULT '0',
  `em7_1` smallint unsigned NOT NULL DEFAULT '0',
  `em7_2` smallint unsigned NOT NULL DEFAULT '0',
  `em7_3` smallint unsigned NOT NULL DEFAULT '0',
  `em7_4` smallint unsigned NOT NULL DEFAULT '0',
  `em7_5` smallint unsigned NOT NULL DEFAULT '0',
  `VerifiedBuild` int DEFAULT NULL,
  PRIMARY KEY (`ID`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb4 COLLATE=utf8mb4_unicode_ci;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `npc_text`
--

LOCK TABLES `npc_text` WRITE;
/*!40000 ALTER TABLE `npc_text` DISABLE KEYS */;
/*!40000 ALTER TABLE `npc_text` ENABLE KEYS */;
UNLOCK TABLES;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2026-06-01 22:40:14
