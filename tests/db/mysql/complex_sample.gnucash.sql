-- MySQL dump 10.13  Distrib 5.5.62, for Win64 (AMD64)
--
-- Host: localhost    Database: complex_sample.gnucash
-- ------------------------------------------------------
-- Server version	8.0.25

/*!40101 SET @OLD_CHARACTER_SET_CLIENT=@@CHARACTER_SET_CLIENT */;
/*!40101 SET @OLD_CHARACTER_SET_RESULTS=@@CHARACTER_SET_RESULTS */;
/*!40101 SET @OLD_COLLATION_CONNECTION=@@COLLATION_CONNECTION */;
/*!40101 SET NAMES utf8 */;
/*!40103 SET @OLD_TIME_ZONE=@@TIME_ZONE */;
/*!40103 SET TIME_ZONE='+00:00' */;
/*!40014 SET @OLD_UNIQUE_CHECKS=@@UNIQUE_CHECKS, UNIQUE_CHECKS=0 */;
/*!40014 SET @OLD_FOREIGN_KEY_CHECKS=@@FOREIGN_KEY_CHECKS, FOREIGN_KEY_CHECKS=0 */;
/*!40101 SET @OLD_SQL_MODE=@@SQL_MODE, SQL_MODE='NO_AUTO_VALUE_ON_ZERO' */;
/*!40111 SET @OLD_SQL_NOTES=@@SQL_NOTES, SQL_NOTES=0 */;

--
-- Table structure for table `accounts`
--

DROP TABLE IF EXISTS `accounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `accounts` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `account_type` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `commodity_guid` varchar(32) DEFAULT NULL,
  `commodity_scu` int NOT NULL,
  `non_std_scu` int NOT NULL,
  `parent_guid` varchar(32) DEFAULT NULL,
  `code` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `description` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `hidden` int DEFAULT NULL,
  `placeholder` int DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `accounts`
--

LOCK TABLES `accounts` WRITE;
/*!40000 ALTER TABLE `accounts` DISABLE KEYS */;
INSERT INTO `accounts` VALUES ('00622dda21937b29e494179de5013f82','Root Account','ROOT',NULL,0,0,NULL,'','',0,0),('01f6b1417935528fdc97ac2e130a150c','Broker','ASSET','5f586908098232e67edb1371408bfaa8',100,0,'fcd795021c976ba75621ec39e75f6214','','',0,1),('0ccab772d0d16a3e1eaf42cd53f891e5','FOO','TRADING','069410ffec45a41a218bba474093d466',10000,0,'f2b78b76f6093ba04204d972a7ce42d6','','',0,0),('1305127e63737f8c39afb49b5bbeca7a','Opening Balances - EUR','EQUITY','346629655191dcf59a7e2c2a85b70f69',100,0,'8056f425ecdc352cf6039e3a0d0d1e6c','','',0,0),('1c089803052e85f5c6d8e786057dbaee','Foo stock','STOCK','069410ffec45a41a218bba474093d466',10000,0,'01f6b1417935528fdc97ac2e130a150c','','',0,0),('3bc319753945b6dba3e1928abed49e35','Current','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'fcd795021c976ba75621ec39e75f6214','','',0,1),('5b5100d58bcc030d2f7828d897fda62e','Fixed','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'fcd795021c976ba75621ec39e75f6214','','',0,0),('6bbc8f20544452cac1637fb9a9b851bb','Income','INCOME','346629655191dcf59a7e2c2a85b70f69',100,0,'00622dda21937b29e494179de5013f82','','',0,0),('7894e9c3e955f5eaa9689d16ed775660','EUR5','TRADING','346629655191dcf59a7e2c2a85b70f69',100,0,'e97809296efb670cff4f92138bd69ec8','','',0,0),('8056f425ecdc352cf6039e3a0d0d1e6c','Equity','EQUITY','346629655191dcf59a7e2c2a85b70f69',100,0,'00622dda21937b29e494179de5013f82','','',0,0),('93fc043c3062aaa1297b30e543d2cd0d','Cash','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'3bc319753945b6dba3e1928abed49e35','','',0,0),('96ed7a45459fb5fe570e48fcd46f05d0','Liability','LIABILITY','346629655191dcf59a7e2c2a85b70f69',100,0,'00622dda21937b29e494179de5013f82','','',0,0),('a1dd9e8118bff87db289482bceebfea9','Savings','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'3bc319753945b6dba3e1928abed49e35','','',0,0),('a3dc764fea53b709af7fcead6470da43','House','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'5b5100d58bcc030d2f7828d897fda62e','','',0,0),('adc619f0ac7fa27d5768bfd73ecbc01e','Checking','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'3bc319753945b6dba3e1928abed49e35','','',0,0),('af88d386d44b14acf244362b85ccaf4c','Expense','EXPENSE','346629655191dcf59a7e2c2a85b70f69',100,0,'00622dda21937b29e494179de5013f82','','',0,0),('bb43218d7d95b1fe062a731d1aa7e9e2','Mouvements','TRADING','069410ffec45a41a218bba474093d466',10000,0,'00622dda21937b29e494179de5013f82','','',0,1),('e97809296efb670cff4f92138bd69ec8','CURRENCY','TRADING','346629655191dcf59a7e2c2a85b70f69',100,0,'bb43218d7d95b1fe062a731d1aa7e9e2','','',0,1),('f2b78b76f6093ba04204d972a7ce42d6','NASDAQ','TRADING','069410ffec45a41a218bba474093d466',10000,0,'bb43218d7d95b1fe062a731d1aa7e9e2','','',0,1),('f6c0cd00ec04169a44f170181882adab','Template Root','ROOT',NULL,0,0,NULL,'','',0,0),('fcd795021c976ba75621ec39e75f6214','Asset','ASSET','346629655191dcf59a7e2c2a85b70f69',100,0,'00622dda21937b29e494179de5013f82','','',0,1);
/*!40000 ALTER TABLE `accounts` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `billterms`
--

DROP TABLE IF EXISTS `billterms`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `billterms` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `description` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `refcount` int NOT NULL,
  `invisible` int NOT NULL,
  `parent` varchar(32) DEFAULT NULL,
  `type` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `duedays` int DEFAULT NULL,
  `discountdays` int DEFAULT NULL,
  `discount_num` bigint DEFAULT NULL,
  `discount_denom` bigint DEFAULT NULL,
  `cutoff` int DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `billterms`
--

LOCK TABLES `billterms` WRITE;
/*!40000 ALTER TABLE `billterms` DISABLE KEYS */;
/*!40000 ALTER TABLE `billterms` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `books`
--

DROP TABLE IF EXISTS `books`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `books` (
  `guid` varchar(32) NOT NULL,
  `root_account_guid` varchar(32) NOT NULL,
  `root_template_guid` varchar(32) NOT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `books`
--

LOCK TABLES `books` WRITE;
/*!40000 ALTER TABLE `books` DISABLE KEYS */;
INSERT INTO `books` VALUES ('7d4ef4044fd30f41d08914a8174c2f5b','00622dda21937b29e494179de5013f82','f6c0cd00ec04169a44f170181882adab');
/*!40000 ALTER TABLE `books` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `budget_amounts`
--

DROP TABLE IF EXISTS `budget_amounts`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `budget_amounts` (
  `id` int NOT NULL AUTO_INCREMENT,
  `budget_guid` varchar(32) NOT NULL,
  `account_guid` varchar(32) NOT NULL,
  `period_num` int NOT NULL,
  `amount_num` bigint NOT NULL,
  `amount_denom` bigint NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `budget_amounts`
--

LOCK TABLES `budget_amounts` WRITE;
/*!40000 ALTER TABLE `budget_amounts` DISABLE KEYS */;
/*!40000 ALTER TABLE `budget_amounts` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `budgets`
--

DROP TABLE IF EXISTS `budgets`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `budgets` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `description` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `num_periods` int NOT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `budgets`
--

LOCK TABLES `budgets` WRITE;
/*!40000 ALTER TABLE `budgets` DISABLE KEYS */;
/*!40000 ALTER TABLE `budgets` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `commodities`
--

DROP TABLE IF EXISTS `commodities`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `commodities` (
  `guid` varchar(32) NOT NULL,
  `namespace` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `mnemonic` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `fullname` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `cusip` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `fraction` int NOT NULL,
  `quote_flag` int NOT NULL,
  `quote_source` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `quote_tz` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `commodities`
--

LOCK TABLES `commodities` WRITE;
/*!40000 ALTER TABLE `commodities` DISABLE KEYS */;
INSERT INTO `commodities` VALUES ('069410ffec45a41a218bba474093d466','NASDAQ','FOO','Foo Inc','',10000,0,NULL,''),('1e5d65e2726a5d4595741cb204992991','CURRENCY','USD','US Dollar','840',100,0,'currency',''),('346629655191dcf59a7e2c2a85b70f69','CURRENCY','EUR','Euro','978',100,1,'currency',''),('5f586908098232e67edb1371408bfaa8','CURRENCY','AED','UAE Dirham','784',100,1,'currency',''),('d821d6776fde9f7c2d01b67876406fd3','CURRENCY','ADF','Andorran Franc','950',100,0,'currency','');
/*!40000 ALTER TABLE `commodities` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `customers`
--

DROP TABLE IF EXISTS `customers`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `customers` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `notes` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `active` int NOT NULL,
  `discount_num` bigint NOT NULL,
  `discount_denom` bigint NOT NULL,
  `credit_num` bigint NOT NULL,
  `credit_denom` bigint NOT NULL,
  `currency` varchar(32) NOT NULL,
  `tax_override` int NOT NULL,
  `addr_name` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr1` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr2` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr3` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr4` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_phone` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_fax` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_email` varchar(256) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_name` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_addr1` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_addr2` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_addr3` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_addr4` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_phone` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_fax` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `shipaddr_email` varchar(256) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `terms` varchar(32) DEFAULT NULL,
  `tax_included` int DEFAULT NULL,
  `taxtable` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `customers`
--

LOCK TABLES `customers` WRITE;
/*!40000 ALTER TABLE `customers` DISABLE KEYS */;
/*!40000 ALTER TABLE `customers` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `employees`
--

DROP TABLE IF EXISTS `employees`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `employees` (
  `guid` varchar(32) NOT NULL,
  `username` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `language` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `acl` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `active` int NOT NULL,
  `currency` varchar(32) NOT NULL,
  `ccard_guid` varchar(32) DEFAULT NULL,
  `workday_num` bigint NOT NULL,
  `workday_denom` bigint NOT NULL,
  `rate_num` bigint NOT NULL,
  `rate_denom` bigint NOT NULL,
  `addr_name` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr1` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr2` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr3` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr4` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_phone` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_fax` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_email` varchar(256) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `employees`
--

LOCK TABLES `employees` WRITE;
/*!40000 ALTER TABLE `employees` DISABLE KEYS */;
/*!40000 ALTER TABLE `employees` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `entries`
--

DROP TABLE IF EXISTS `entries`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `entries` (
  `guid` varchar(32) NOT NULL,
  `date` datetime NOT NULL DEFAULT '1970-01-01 00:00:00',
  `date_entered` datetime DEFAULT '1970-01-01 00:00:00',
  `description` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `action` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `notes` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `quantity_num` bigint DEFAULT NULL,
  `quantity_denom` bigint DEFAULT NULL,
  `i_acct` varchar(32) DEFAULT NULL,
  `i_price_num` bigint DEFAULT NULL,
  `i_price_denom` bigint DEFAULT NULL,
  `i_discount_num` bigint DEFAULT NULL,
  `i_discount_denom` bigint DEFAULT NULL,
  `invoice` varchar(32) DEFAULT NULL,
  `i_disc_type` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `i_disc_how` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `i_taxable` int DEFAULT NULL,
  `i_taxincluded` int DEFAULT NULL,
  `i_taxtable` varchar(32) DEFAULT NULL,
  `b_acct` varchar(32) DEFAULT NULL,
  `b_price_num` bigint DEFAULT NULL,
  `b_price_denom` bigint DEFAULT NULL,
  `bill` varchar(32) DEFAULT NULL,
  `b_taxable` int DEFAULT NULL,
  `b_taxincluded` int DEFAULT NULL,
  `b_taxtable` varchar(32) DEFAULT NULL,
  `b_paytype` int DEFAULT NULL,
  `billable` int DEFAULT NULL,
  `billto_type` int DEFAULT NULL,
  `billto_guid` varchar(32) DEFAULT NULL,
  `order_guid` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `entries`
--

LOCK TABLES `entries` WRITE;
/*!40000 ALTER TABLE `entries` DISABLE KEYS */;
/*!40000 ALTER TABLE `entries` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `gnclock`
--

DROP TABLE IF EXISTS `gnclock`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `gnclock` (
  `Hostname` varchar(255) DEFAULT NULL,
  `PID` int DEFAULT NULL
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `gnclock`
--

LOCK TABLES `gnclock` WRITE;
/*!40000 ALTER TABLE `gnclock` DISABLE KEYS */;
/*!40000 ALTER TABLE `gnclock` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `invoices`
--

DROP TABLE IF EXISTS `invoices`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `invoices` (
  `guid` varchar(32) NOT NULL,
  `id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `date_opened` datetime DEFAULT '1970-01-01 00:00:00',
  `date_posted` datetime DEFAULT '1970-01-01 00:00:00',
  `notes` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `active` int NOT NULL,
  `currency` varchar(32) NOT NULL,
  `owner_type` int DEFAULT NULL,
  `owner_guid` varchar(32) DEFAULT NULL,
  `terms` varchar(32) DEFAULT NULL,
  `billing_id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `post_txn` varchar(32) DEFAULT NULL,
  `post_lot` varchar(32) DEFAULT NULL,
  `post_acc` varchar(32) DEFAULT NULL,
  `billto_type` int DEFAULT NULL,
  `billto_guid` varchar(32) DEFAULT NULL,
  `charge_amt_num` bigint DEFAULT NULL,
  `charge_amt_denom` bigint DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `invoices`
--

LOCK TABLES `invoices` WRITE;
/*!40000 ALTER TABLE `invoices` DISABLE KEYS */;
/*!40000 ALTER TABLE `invoices` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `jobs`
--

DROP TABLE IF EXISTS `jobs`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `jobs` (
  `guid` varchar(32) NOT NULL,
  `id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `reference` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `active` int NOT NULL,
  `owner_type` int DEFAULT NULL,
  `owner_guid` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `jobs`
--

LOCK TABLES `jobs` WRITE;
/*!40000 ALTER TABLE `jobs` DISABLE KEYS */;
/*!40000 ALTER TABLE `jobs` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `lots`
--

DROP TABLE IF EXISTS `lots`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `lots` (
  `guid` varchar(32) NOT NULL,
  `account_guid` varchar(32) DEFAULT NULL,
  `is_closed` int NOT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `lots`
--

LOCK TABLES `lots` WRITE;
/*!40000 ALTER TABLE `lots` DISABLE KEYS */;
/*!40000 ALTER TABLE `lots` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `orders`
--

DROP TABLE IF EXISTS `orders`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `orders` (
  `guid` varchar(32) NOT NULL,
  `id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `notes` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `reference` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `active` int NOT NULL,
  `date_opened` datetime NOT NULL DEFAULT '1970-01-01 00:00:00',
  `date_closed` datetime NOT NULL DEFAULT '1970-01-01 00:00:00',
  `owner_type` int NOT NULL,
  `owner_guid` varchar(32) NOT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `orders`
--

LOCK TABLES `orders` WRITE;
/*!40000 ALTER TABLE `orders` DISABLE KEYS */;
/*!40000 ALTER TABLE `orders` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `prices`
--

DROP TABLE IF EXISTS `prices`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `prices` (
  `guid` varchar(32) NOT NULL,
  `commodity_guid` varchar(32) NOT NULL,
  `currency_guid` varchar(32) NOT NULL,
  `date` datetime NOT NULL DEFAULT '1970-01-01 00:00:00',
  `source` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `type` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `value_num` bigint NOT NULL,
  `value_denom` bigint NOT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `prices`
--

LOCK TABLES `prices` WRITE;
/*!40000 ALTER TABLE `prices` DISABLE KEYS */;
INSERT INTO `prices` VALUES ('0d6684f44fb018e882de76094ed9c433','d821d6776fde9f7c2d01b67876406fd3','5f586908098232e67edb1371408bfaa8','2018-02-20 23:00:00','user:price-editor','unknown',3,2),('715ad31cb272437fc883d658f732a8eb','1e5d65e2726a5d4595741cb204992991','346629655191dcf59a7e2c2a85b70f69','2018-02-20 23:00:00','user:price-editor','unknown',7,5),('831ce95a146243a0b66895151538990a','5f586908098232e67edb1371408bfaa8','346629655191dcf59a7e2c2a85b70f69','2017-05-08 16:00:00','user:price-editor','last',1,2),('86a98448a82e9012cad82ac677eb86df','069410ffec45a41a218bba474093d466','5f586908098232e67edb1371408bfaa8','2018-02-20 23:00:00','user:price-editor','unknown',9,10),('c266e6ce9bdf8832bf88360df524669e','346629655191dcf59a7e2c2a85b70f69','5f586908098232e67edb1371408bfaa8','2018-02-20 23:00:00','user:price-editor','unknown',10,9);
/*!40000 ALTER TABLE `prices` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `recurrences`
--

DROP TABLE IF EXISTS `recurrences`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `recurrences` (
  `id` int NOT NULL AUTO_INCREMENT,
  `obj_guid` varchar(32) NOT NULL,
  `recurrence_mult` int NOT NULL,
  `recurrence_period_type` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `recurrence_period_start` date NOT NULL,
  `recurrence_weekend_adjust` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `recurrences`
--

LOCK TABLES `recurrences` WRITE;
/*!40000 ALTER TABLE `recurrences` DISABLE KEYS */;
/*!40000 ALTER TABLE `recurrences` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `schedxactions`
--

DROP TABLE IF EXISTS `schedxactions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `schedxactions` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `enabled` int NOT NULL,
  `start_date` date DEFAULT NULL,
  `end_date` date DEFAULT NULL,
  `last_occur` date DEFAULT NULL,
  `num_occur` int NOT NULL,
  `rem_occur` int NOT NULL,
  `auto_create` int NOT NULL,
  `auto_notify` int NOT NULL,
  `adv_creation` int NOT NULL,
  `adv_notify` int NOT NULL,
  `instance_count` int NOT NULL,
  `template_act_guid` varchar(32) NOT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `schedxactions`
--

LOCK TABLES `schedxactions` WRITE;
/*!40000 ALTER TABLE `schedxactions` DISABLE KEYS */;
/*!40000 ALTER TABLE `schedxactions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `slots`
--

DROP TABLE IF EXISTS `slots`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `slots` (
  `id` int NOT NULL AUTO_INCREMENT,
  `obj_guid` varchar(32) NOT NULL,
  `name` varchar(4096) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `slot_type` int NOT NULL,
  `int64_val` bigint DEFAULT NULL,
  `string_val` varchar(4096) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `double_val` double DEFAULT NULL,
  `timespec_val` datetime DEFAULT '1970-01-01 00:00:00',
  `guid_val` varchar(32) DEFAULT NULL,
  `numeric_val_num` bigint DEFAULT NULL,
  `numeric_val_denom` bigint DEFAULT NULL,
  `gdate_val` date DEFAULT NULL,
  PRIMARY KEY (`id`),
  KEY `slots_guid_index` (`obj_guid`)
) ENGINE=InnoDB AUTO_INCREMENT=15 DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `slots`
--

LOCK TABLES `slots` WRITE;
/*!40000 ALTER TABLE `slots` DISABLE KEYS */;
INSERT INTO `slots` VALUES (1,'7d4ef4044fd30f41d08914a8174c2f5b','features',9,0,NULL,NULL,'1970-01-01 00:00:00','28428f5b949346a4ac2e6f27fe5c4990',0,1,NULL),(2,'28428f5b949346a4ac2e6f27fe5c4990','features/ISO-8601 formatted date strings in SQLite3 databases.',4,0,'Use ISO formatted date-time strings in SQLite3 databases (requires at least GnuCash 2.6.20)',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(3,'7d4ef4044fd30f41d08914a8174c2f5b','options',9,0,NULL,NULL,'1970-01-01 00:00:00','349a6c8eaede44f184006a7582dac8a7',0,1,NULL),(4,'349a6c8eaede44f184006a7582dac8a7','options/Accounts',9,0,NULL,NULL,'1970-01-01 00:00:00','6ba23dcc11574117b50eea3336cb3387',0,1,NULL),(5,'6ba23dcc11574117b50eea3336cb3387','options/Accounts/Use Trading Accounts',4,0,'t',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(6,'349a6c8eaede44f184006a7582dac8a7','options/Budgeting',9,0,NULL,NULL,'1970-01-01 00:00:00','99fd48ba1ba547138b0ee7c20a356a77',0,1,NULL),(7,'7d4ef4044fd30f41d08914a8174c2f5b','remove-color-not-set-slots',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(8,'fcd795021c976ba75621ec39e75f6214','placeholder',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(9,'3bc319753945b6dba3e1928abed49e35','placeholder',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(10,'01f6b1417935528fdc97ac2e130a150c','placeholder',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(11,'bb43218d7d95b1fe062a731d1aa7e9e2','placeholder',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(12,'f2b78b76f6093ba04204d972a7ce42d6','placeholder',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(13,'e97809296efb670cff4f92138bd69ec8','placeholder',4,0,'true',NULL,'1970-01-01 00:00:00',NULL,0,1,NULL),(14,'0fac88d8412f46d5e16beb87aee0fd22','date-posted',10,0,NULL,NULL,'1970-01-01 00:00:00',NULL,0,1,'2018-02-20');
/*!40000 ALTER TABLE `slots` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `splits`
--

DROP TABLE IF EXISTS `splits`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `splits` (
  `guid` varchar(32) NOT NULL,
  `tx_guid` varchar(32) NOT NULL,
  `account_guid` varchar(32) NOT NULL,
  `memo` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `action` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `reconcile_state` varchar(1) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `reconcile_date` datetime DEFAULT '1970-01-01 00:00:00',
  `value_num` bigint NOT NULL,
  `value_denom` bigint NOT NULL,
  `quantity_num` bigint NOT NULL,
  `quantity_denom` bigint NOT NULL,
  `lot_guid` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`guid`),
  KEY `splits_tx_guid_index` (`tx_guid`),
  KEY `splits_account_guid_index` (`account_guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `splits`
--

LOCK TABLES `splits` WRITE;
/*!40000 ALTER TABLE `splits` DISABLE KEYS */;
INSERT INTO `splits` VALUES ('1e612f650eb598d9e803902b6aca73e3','6c8876003c4a6026e38e3afb67d6f2b1','6bbc8f20544452cac1637fb9a9b851bb','','','n','1970-01-01 00:00:00',-15000,100,-15000,100,NULL),('3142f6ebed4469f9a692dfa69a0080f5','325537f4f0fadfd9ffb6aad3cd18e360','af88d386d44b14acf244362b85ccaf4c','interest','','n','1970-01-01 00:00:00',3000,100,3000,100,NULL),('3a53ab1f73cb74d9882dd14b7d6a4a78','5be11c009a88b4198aa65ee878ddf99d','96ed7a45459fb5fe570e48fcd46f05d0','','','n','1970-01-01 00:00:00',-100000,100,-100000,100,NULL),('42336a5732d54b22fa080be3b50ff5b7','0fac88d8412f46d5e16beb87aee0fd22','93fc043c3062aaa1297b30e543d2cd0d','','','n','1970-01-01 00:00:00',10000,100,10000,100,NULL),('47c7fdca3bc04d6ad734eefc2794ddfe','325537f4f0fadfd9ffb6aad3cd18e360','adc619f0ac7fa27d5768bfd73ecbc01e','monthly payment','','n','1970-01-01 00:00:00',-13000,100,-13000,100,NULL),('486341b68457b037ecd75dcdcbf64bc5','0fac88d8412f46d5e16beb87aee0fd22','adc619f0ac7fa27d5768bfd73ecbc01e','','','n','1970-01-01 00:00:00',-10000,100,-10000,100,NULL),('4f78ef3650c38115abf60365f9484b45','7b3a2df3645b3e490121814c8ad7146d','a1dd9e8118bff87db289482bceebfea9','','','n','1970-01-01 00:00:00',250000,100,250000,100,NULL),('6ca852f33ee5415d22dfc132f5c5ec09','325537f4f0fadfd9ffb6aad3cd18e360','96ed7a45459fb5fe570e48fcd46f05d0','capital','','n','1970-01-01 00:00:00',10000,100,10000,100,NULL),('714c69b235d450f11e4add7077b4cde7','5be11c009a88b4198aa65ee878ddf99d','adc619f0ac7fa27d5768bfd73ecbc01e','','','n','1970-01-01 00:00:00',100000,100,100000,100,NULL),('76cd1180d563db608495c710d2774c9a','59d9b60221a4615e74f8e489dceff58b','a3dc764fea53b709af7fcead6470da43','','','n','1970-01-01 00:00:00',2000000,100,2000000,100,NULL),('85d451d09db85fb97cea6fe2e876d566','a5924cd14525c307cc5862c97361b031','0ccab772d0d16a3e1eaf42cd53f891e5','','','n','1970-01-01 00:00:00',-120000,100,-1300000,10000,NULL),('afcba9bb98422e5ca64931a5efd86025','7b3a2df3645b3e490121814c8ad7146d','1305127e63737f8c39afb49b5bbeca7a','','','n','1970-01-01 00:00:00',-250000,100,-250000,100,NULL),('b3c4df045f2a19332e143cd83a7e013f','2d249b7d3fc8efe17865873b97748f50','a1dd9e8118bff87db289482bceebfea9','','','n','1970-01-01 00:00:00',250000,100,250000,100,NULL),('b8387081c3d60310465a5d316fb6745a','6294086c678a584e7ce184b523699f6c','af88d386d44b14acf244362b85ccaf4c','','','n','1970-01-01 00:00:00',20000,100,20000,100,NULL),('bae8bc3a2b29812691016e61c425fda5','e774d4b1fbd3c8c853b2c45a1d74f5f9','a1dd9e8118bff87db289482bceebfea9','','','n','1970-01-01 00:00:00',-25000,100,-25000,100,NULL),('c8eb859a307babbb2fdf66871cf4c0a8','59d9b60221a4615e74f8e489dceff58b','96ed7a45459fb5fe570e48fcd46f05d0','','','n','1970-01-01 00:00:00',-2000000,100,-2000000,100,NULL),('d240433d6812fc634c21200fb559d07f','2d249b7d3fc8efe17865873b97748f50','1305127e63737f8c39afb49b5bbeca7a','','','n','1970-01-01 00:00:00',-250000,100,-250000,100,NULL),('d96c36fed7324332bfeff80e52cd9892','6294086c678a584e7ce184b523699f6c','adc619f0ac7fa27d5768bfd73ecbc01e','','','n','1970-01-01 00:00:00',-20000,100,-20000,100,NULL),('de832fe97e37811a7fff7e28b3a43425','6c8876003c4a6026e38e3afb67d6f2b1','93fc043c3062aaa1297b30e543d2cd0d','','','n','1970-01-01 00:00:00',15000,100,15000,100,NULL),('e055799a549308621a16c715c870e29a','a5924cd14525c307cc5862c97361b031','7894e9c3e955f5eaa9689d16ed775660','','','n','1970-01-01 00:00:00',120000,100,120000,100,NULL),('e07539176a70222b16369bd246b174b9','a5924cd14525c307cc5862c97361b031','1c089803052e85f5c6d8e786057dbaee','','','n','1970-01-01 00:00:00',120000,100,1300000,10000,NULL),('ea72c01205e1f619aa278930648b060e','e774d4b1fbd3c8c853b2c45a1d74f5f9','adc619f0ac7fa27d5768bfd73ecbc01e','','','n','1970-01-01 00:00:00',25000,100,25000,100,NULL),('eb832677cb2a013fcc4140316ccb3323','a5b50b770ee5e01361c49300aedb5e0f','af88d386d44b14acf244362b85ccaf4c','','','n','1970-01-01 00:00:00',3000,100,3000,100,NULL),('f32095185808aff854fc020ee2eedb5d','a5b50b770ee5e01361c49300aedb5e0f','93fc043c3062aaa1297b30e543d2cd0d','','','n','1970-01-01 00:00:00',-3000,100,-3000,100,NULL),('f5f1b767b38439eff788bed8fca8d1c2','a5924cd14525c307cc5862c97361b031','a1dd9e8118bff87db289482bceebfea9','','','n','1970-01-01 00:00:00',-120000,100,-120000,100,NULL);
/*!40000 ALTER TABLE `splits` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `taxtable_entries`
--

DROP TABLE IF EXISTS `taxtable_entries`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `taxtable_entries` (
  `id` int NOT NULL AUTO_INCREMENT,
  `taxtable` varchar(32) NOT NULL,
  `account` varchar(32) NOT NULL,
  `amount_num` bigint NOT NULL,
  `amount_denom` bigint NOT NULL,
  `type` int NOT NULL,
  PRIMARY KEY (`id`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `taxtable_entries`
--

LOCK TABLES `taxtable_entries` WRITE;
/*!40000 ALTER TABLE `taxtable_entries` DISABLE KEYS */;
/*!40000 ALTER TABLE `taxtable_entries` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `taxtables`
--

DROP TABLE IF EXISTS `taxtables`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `taxtables` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(50) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `refcount` bigint NOT NULL,
  `invisible` int NOT NULL,
  `parent` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `taxtables`
--

LOCK TABLES `taxtables` WRITE;
/*!40000 ALTER TABLE `taxtables` DISABLE KEYS */;
/*!40000 ALTER TABLE `taxtables` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `transactions`
--

DROP TABLE IF EXISTS `transactions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `transactions` (
  `guid` varchar(32) NOT NULL,
  `currency_guid` varchar(32) NOT NULL,
  `num` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `post_date` datetime DEFAULT '1970-01-01 00:00:00',
  `enter_date` datetime DEFAULT '1970-01-01 00:00:00',
  `description` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  PRIMARY KEY (`guid`),
  KEY `tx_post_date_index` (`post_date`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `transactions`
--

LOCK TABLES `transactions` WRITE;
/*!40000 ALTER TABLE `transactions` DISABLE KEYS */;
INSERT INTO `transactions` VALUES ('0fac88d8412f46d5e16beb87aee0fd22','346629655191dcf59a7e2c2a85b70f69','','2018-02-20 10:59:00','2018-02-20 21:12:49','Transfer current'),('2d249b7d3fc8efe17865873b97748f50','346629655191dcf59a7e2c2a85b70f69','','2018-02-21 10:59:00','2018-02-21 07:25:53','Opening Balance'),('325537f4f0fadfd9ffb6aad3cd18e360','346629655191dcf59a7e2c2a85b70f69','','2014-12-24 10:59:00','2014-12-25 10:11:26','loan payment'),('59d9b60221a4615e74f8e489dceff58b','346629655191dcf59a7e2c2a85b70f69','','2018-02-20 10:59:00','2018-02-20 21:14:36','house load'),('5be11c009a88b4198aa65ee878ddf99d','346629655191dcf59a7e2c2a85b70f69','','2014-12-24 10:59:00','2014-12-25 10:07:30','initial load'),('6294086c678a584e7ce184b523699f6c','346629655191dcf59a7e2c2a85b70f69','','2014-12-24 10:59:00','2014-12-25 10:08:08','expense 1'),('6c8876003c4a6026e38e3afb67d6f2b1','346629655191dcf59a7e2c2a85b70f69','','2014-12-24 10:59:00','2014-12-25 10:08:15','income 1'),('7b3a2df3645b3e490121814c8ad7146d','346629655191dcf59a7e2c2a85b70f69','','2018-02-20 10:59:00','2018-02-20 21:13:50','Opening Balance'),('a5924cd14525c307cc5862c97361b031','346629655191dcf59a7e2c2a85b70f69','','2018-02-21 10:59:00','2018-02-21 06:58:44','buy foo'),('a5b50b770ee5e01361c49300aedb5e0f','346629655191dcf59a7e2c2a85b70f69','','2018-02-20 10:59:00','2018-02-20 21:13:01','Purchase'),('e774d4b1fbd3c8c853b2c45a1d74f5f9','346629655191dcf59a7e2c2a85b70f69','','2018-02-20 10:59:00','2018-02-20 21:13:30','transfer intra');
/*!40000 ALTER TABLE `transactions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `vendors`
--

DROP TABLE IF EXISTS `vendors`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `vendors` (
  `guid` varchar(32) NOT NULL,
  `name` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `id` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `notes` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `currency` varchar(32) NOT NULL,
  `active` int NOT NULL,
  `tax_override` int NOT NULL,
  `addr_name` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr1` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr2` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr3` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_addr4` varchar(1024) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_phone` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_fax` varchar(128) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `addr_email` varchar(256) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `terms` varchar(32) DEFAULT NULL,
  `tax_inc` varchar(2048) CHARACTER SET utf8 COLLATE utf8_general_ci DEFAULT NULL,
  `tax_table` varchar(32) DEFAULT NULL,
  PRIMARY KEY (`guid`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `vendors`
--

LOCK TABLES `vendors` WRITE;
/*!40000 ALTER TABLE `vendors` DISABLE KEYS */;
/*!40000 ALTER TABLE `vendors` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Table structure for table `versions`
--

DROP TABLE IF EXISTS `versions`;
/*!40101 SET @saved_cs_client     = @@character_set_client */;
/*!40101 SET character_set_client = utf8 */;
CREATE TABLE `versions` (
  `table_name` varchar(50) CHARACTER SET utf8 COLLATE utf8_general_ci NOT NULL,
  `table_version` int NOT NULL,
  PRIMARY KEY (`table_name`)
) ENGINE=InnoDB DEFAULT CHARSET=utf8mb3;
/*!40101 SET character_set_client = @saved_cs_client */;

--
-- Dumping data for table `versions`
--

LOCK TABLES `versions` WRITE;
/*!40000 ALTER TABLE `versions` DISABLE KEYS */;
INSERT INTO `versions` VALUES ('accounts',1),('billterms',2),('books',1),('budgets',1),('budget_amounts',1),('commodities',1),('customers',2),('employees',2),('entries',4),('Gnucash',3000011),('Gnucash-Resave',19920),('invoices',4),('jobs',1),('lots',2),('orders',1),('prices',3),('recurrences',2),('schedxactions',1),('slots',4),('splits',5),('taxtables',2),('taxtable_entries',3),('transactions',4),('vendors',1);
/*!40000 ALTER TABLE `versions` ENABLE KEYS */;
UNLOCK TABLES;

--
-- Dumping routines for database 'complex_sample.gnucash'
--
/*!40103 SET TIME_ZONE=@OLD_TIME_ZONE */;

/*!40101 SET SQL_MODE=@OLD_SQL_MODE */;
/*!40014 SET FOREIGN_KEY_CHECKS=@OLD_FOREIGN_KEY_CHECKS */;
/*!40014 SET UNIQUE_CHECKS=@OLD_UNIQUE_CHECKS */;
/*!40101 SET CHARACTER_SET_CLIENT=@OLD_CHARACTER_SET_CLIENT */;
/*!40101 SET CHARACTER_SET_RESULTS=@OLD_CHARACTER_SET_RESULTS */;
/*!40101 SET COLLATION_CONNECTION=@OLD_COLLATION_CONNECTION */;
/*!40111 SET SQL_NOTES=@OLD_SQL_NOTES */;

-- Dump completed on 2021-05-31 11:04:04
