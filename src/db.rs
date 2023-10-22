use rusqlite::{Connection, Result};
use std::path::PathBuf;

/*
 * table data {
 *    id TEXT PRIMARY KEY,
 *    data BLOB
 * }
 * table notifications {
 *    id TEXT PRIMARY KEY,
 * }
 */

pub fn connect(db_path: PathBuf) -> Result<Connection> {
    let conn = Connection::open(db_path)?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS data (
            id TEXT PRIMARY KEY,
            data BLOB
        )",
        [],
    )?;

    conn.execute(
        "CREATE TABLE IF NOT EXISTS notifications (
            id TEXT PRIMARY KEY
        )",
        [],
    )?;

    Ok(conn)
}

pub fn get_data<T>(conn: &Connection, id: &str) -> Result<Option<T>>
where
    T: serde::de::DeserializeOwned,
{
    let mut stmt = conn.prepare("SELECT data FROM data WHERE id = ?")?;
    let mut rows = stmt.query([id])?;

    if let Some(row) = rows.next()? {
        let data: Vec<u8> = row.get(0)?;
        let data = bincode::deserialize(&data).unwrap();
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

pub fn set_data<T: ?Sized>(conn: &Connection, id: &str, data: &T) -> Result<()>
where
    T: serde::Serialize,
{
    let data = bincode::serialize(data).unwrap();
    conn.execute(
        "INSERT INTO data (id, data) VALUES (?, ?) ON CONFLICT (id) DO UPDATE SET data = ?",
        rusqlite::params![id, data, data],
    )?;

    Ok(())
}

pub fn get_data_raw(conn: &Connection, id: &str) -> Result<Option<Vec<u8>>> {
    let mut stmt = conn.prepare("SELECT data FROM data WHERE id = ?")?;
    let mut rows = stmt.query([id])?;

    if let Some(row) = rows.next()? {
        let data: Vec<u8> = row.get(0)?;
        Ok(Some(data))
    } else {
        Ok(None)
    }
}

pub fn set_data_raw(conn: &Connection, id: &str, data: Vec<u8>) -> Result<()> {
    conn.execute(
        "INSERT INTO data (id, data) VALUES (?, ?) ON CONFLICT (id) DO UPDATE SET data = ?",
        rusqlite::params![id, data, data],
    )?;

    Ok(())
}

pub fn get_notifications(conn: &Connection) -> Result<Vec<String>> {
    let mut stmt = conn.prepare("SELECT id FROM notifications")?;
    let mut rows = stmt.query([])?;

    let mut notifications = Vec::new();

    while let Some(row) = rows.next()? {
        let id: String = row.get(0)?;
        notifications.push(id);
    }

    Ok(notifications)
}

pub fn add_notification(conn: &Connection, id: &str) -> Result<()> {
    conn.execute(
        "INSERT INTO notifications (id) VALUES (?) ON CONFLICT (id) DO NOTHING",
        [id],
    )?;

    Ok(())
}
