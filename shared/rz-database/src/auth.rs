//! Requêtes LoginDatabase (`acore_auth`) : comptes, accès GM, bannissements, royaumes.

use anyhow::Result;
use sqlx::{MySqlPool, Row};

use crate::srp6;

#[derive(Clone, Debug)]
pub struct Account {
    pub id: u32,
    pub username: String,
    pub salt: Vec<u8>,
    pub verifier: Vec<u8>,
    pub locked: bool,
    pub last_ip: String,
    pub expansion: u8,
    pub locale: u8,
}

pub async fn find_account(db: &MySqlPool, username: &str) -> Result<Option<Account>> {
    let row = sqlx::query(
        "SELECT id, username, salt, verifier, locked, last_ip, expansion, locale \
         FROM account WHERE username = ?",
    )
    .bind(srp6::normalize(username))
    .fetch_optional(db)
    .await?;
    Ok(row.map(|r| Account {
        id: r.get("id"),
        username: r.get("username"),
        salt: r.get("salt"),
        verifier: r.get("verifier"),
        locked: r.get::<u8, _>("locked") != 0,
        last_ip: r.get("last_ip"),
        expansion: r.get("expansion"),
        locale: r.get("locale"),
    }))
}

/// Équivalent de la commande `.account create` d'AzerothCore.
pub async fn create_account(db: &MySqlPool, username: &str, password: &str, email: &str) -> Result<u32> {
    anyhow::ensure!(!username.is_empty() && username.chars().count() <= 32, "nom de compte invalide (1 à 32 caractères)");
    anyhow::ensure!(password.chars().count() <= 16, "mot de passe trop long (16 caractères max, comme AzerothCore)");
    let (salt, verifier) = srp6::make_registration(username, password);
    let res = sqlx::query(
        "INSERT INTO account (username, salt, verifier, email, reg_mail, joindate) \
         VALUES (?, ?, ?, ?, ?, NOW())",
    )
    .bind(srp6::normalize(username))
    .bind(&salt[..])
    .bind(&verifier[..])
    .bind(email)
    .bind(email)
    .execute(db)
    .await?;
    let id = res.last_insert_id() as u32;
    sqlx::query("INSERT INTO realmcharacters (realmid, acctid, numchars) SELECT id, ?, 0 FROM realmlist")
        .bind(id)
        .execute(db)
        .await?;
    Ok(id)
}

pub async fn set_password(db: &MySqlPool, account_id: u32, username: &str, password: &str) -> Result<()> {
    let (salt, verifier) = srp6::make_registration(username, password);
    sqlx::query("UPDATE account SET salt = ?, verifier = ? WHERE id = ?")
        .bind(&salt[..])
        .bind(&verifier[..])
        .bind(account_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn set_gm_level(db: &MySqlPool, account_id: u32, level: u8, realm_id: i32) -> Result<()> {
    sqlx::query("DELETE FROM account_access WHERE id = ? AND (RealmID = ? OR RealmID = -1)")
        .bind(account_id)
        .bind(realm_id)
        .execute(db)
        .await?;
    if level > 0 {
        sqlx::query("INSERT INTO account_access (id, gmlevel, RealmID, comment) VALUES (?, ?, ?, 'RappelzCore')")
            .bind(account_id)
            .bind(level)
            .bind(realm_id)
            .execute(db)
            .await?;
    }
    Ok(())
}

pub async fn gm_level(db: &MySqlPool, account_id: u32, realm_id: i32) -> Result<u8> {
    let level: Option<u8> = sqlx::query_scalar(
        "SELECT MAX(gmlevel) FROM account_access WHERE id = ? AND (RealmID = ? OR RealmID = -1)",
    )
    .bind(account_id)
    .bind(realm_id)
    .fetch_one(db)
    .await?;
    Ok(level.unwrap_or(0))
}

pub async fn is_banned(db: &MySqlPool, account_id: u32, ip: &str) -> Result<bool> {
    let acc: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM account_banned WHERE id = ? AND active = 1 \
         AND (unbandate > UNIX_TIMESTAMP() OR unbandate = bandate)",
    )
    .bind(account_id)
    .fetch_one(db)
    .await?;
    let ipb: i64 = sqlx::query_scalar(
        "SELECT COUNT(*) FROM ip_banned WHERE ip = ? AND (unbandate > UNIX_TIMESTAMP() OR unbandate = bandate)",
    )
    .bind(ip)
    .fetch_one(db)
    .await?;
    Ok(acc > 0 || ipb > 0)
}

pub async fn record_login(db: &MySqlPool, account_id: u32, ip: &str, session_key: &[u8]) -> Result<()> {
    sqlx::query(
        "UPDATE account SET last_ip = ?, last_login = NOW(), failed_logins = 0, session_key = ?, os = 'Win' WHERE id = ?",
    )
    .bind(ip)
    .bind(session_key)
    .bind(account_id)
    .execute(db)
    .await?;
    Ok(())
}

pub async fn record_failed_login(db: &MySqlPool, username: &str, ip: &str) -> Result<()> {
    sqlx::query("UPDATE account SET failed_logins = failed_logins + 1, last_attempt_ip = ? WHERE username = ?")
        .bind(ip)
        .bind(srp6::normalize(username))
        .execute(db)
        .await?;
    Ok(())
}

pub async fn session_key(db: &MySqlPool, account_id: u32) -> Result<Option<Vec<u8>>> {
    Ok(sqlx::query_scalar("SELECT session_key FROM account WHERE id = ?")
        .bind(account_id)
        .fetch_optional(db)
        .await?
        .flatten())
}

pub async fn set_online(db: &MySqlPool, account_id: u32, realm_id: u32) -> Result<()> {
    sqlx::query("UPDATE account SET online = ? WHERE id = ?")
        .bind(realm_id)
        .bind(account_id)
        .execute(db)
        .await?;
    Ok(())
}

/// Recover stale account flags after an unclean world exit, without touching other realms.
pub async fn reset_realm_online(db: &MySqlPool, realm_id: u32) -> Result<()> {
    anyhow::ensure!(realm_id != 0, "a positive realm ID is required for startup recovery");
    sqlx::query("UPDATE account SET online = 0 WHERE online = ?")
        .bind(realm_id)
        .execute(db)
        .await?;
    Ok(())
}

#[derive(Clone, Debug)]
pub struct Realm {
    pub id: u32,
    pub name: String,
    pub address: String,
    pub local_address: String,
    pub port: u16,
    pub icon: u8,
    pub flag: u8,
    pub allowed_security_level: u8,
    pub population: f32,
}

pub async fn realms(db: &MySqlPool) -> Result<Vec<Realm>> {
    let rows = sqlx::query(
        "SELECT id, name, address, localAddress, port, icon, flag, allowedSecurityLevel, population \
         FROM realmlist WHERE flag <> 3 ORDER BY id",
    )
    .fetch_all(db)
    .await?;
    Ok(rows
        .into_iter()
        .map(|r| Realm {
            id: r.get("id"),
            name: r.get("name"),
            address: r.get("address"),
            local_address: r.get("localAddress"),
            port: r.get("port"),
            icon: r.get("icon"),
            flag: r.get("flag"),
            allowed_security_level: r.get("allowedSecurityLevel"),
            population: r.get("population"),
        })
        .collect())
}

pub async fn realm_character_count(db: &MySqlPool, realm_id: u32, account_id: u32) -> Result<u8> {
    Ok(sqlx::query_scalar("SELECT numchars FROM realmcharacters WHERE realmid = ? AND acctid = ?")
        .bind(realm_id)
        .bind(account_id)
        .fetch_optional(db)
        .await?
        .unwrap_or(0))
}

pub async fn update_realm_character_count(db: &MySqlPool, realm_id: u32, account_id: u32, count: u8) -> Result<()> {
    sqlx::query("REPLACE INTO realmcharacters (realmid, acctid, numchars) VALUES (?, ?, ?)")
        .bind(realm_id)
        .bind(account_id)
        .bind(count)
        .execute(db)
        .await?;
    Ok(())
}

/// Le worldserver met à jour la population et le drapeau « hors ligne » (flag 2) de son royaume.
pub async fn update_realm_status(db: &MySqlPool, realm_id: u32, online: bool, population: f32) -> Result<()> {
    sqlx::query("UPDATE realmlist SET flag = IF(?, flag & ~2, flag | 2), population = ? WHERE id = ?")
        .bind(online)
        .bind(population)
        .bind(realm_id)
        .execute(db)
        .await?;
    Ok(())
}

pub async fn record_uptime(db: &MySqlPool, realm_id: u32, start_time: i64, uptime: u32, max_players: u16) -> Result<()> {
    sqlx::query(
        "REPLACE INTO uptime (realmid, starttime, uptime, maxplayers, revision) VALUES (?, ?, ?, ?, ?)",
    )
    .bind(realm_id)
    .bind(start_time)
    .bind(uptime)
    .bind(max_players)
    .bind(concat!("RappelzCore ", env!("CARGO_PKG_VERSION")))
    .execute(db)
    .await?;
    Ok(())
}
