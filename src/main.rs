use rusqlite::Connection;

pub fn get_ratchet_states(conn: &Connection, salted_addr: &[u8; 16]) -> Result<(), rusqlite::Error> {
	let mut stmt = conn.prepare("SELECT fingerprint, key, count FROM ratchets WHERE peer = ?1")?;
	let mut rows = stmt.query([salted_addr])?;
	Ok(())
}

fn main() {
	let mut conn = Connection::open("./testdb.db3").unwrap();
	conn.execute("
CREATE TABLE IF NOT EXISTS peers (
	salted_addr CHAR(16) PRIMARY KEY,
	short_addr CHAR(16) NOT NULL,
	identity TEXT,
	ip TEXT,
	status CHAR(4)
)
	", ()).unwrap();
	conn.execute("
CREATE TABLE IF NOT EXISTS ratchets (
	salted_fingerprint CHAR(16) PRIMARY KEY,
	fingerprint CHAR(32),
	key CHAR(32) NOT NULL,
	count BIGINT NOT NULL,

	peer CHAR(16) NOT NULL,
	FOREIGN KEY (peer) REFERENCES peers(salted_addr)
		ON DELETE CASCADE
)
	", ()).unwrap();
	conn.execute("
CREATE TABLE IF NOT EXISTS invite_tokens (
	salted_token CHAR(16) PRIMARY KEY,
	token VARCHAR(32) NOT NULL,
    expiry_time BIGINT NOT NULL,

	salted_fingerprint CHAR(16) UNIQUE NOT NULL,
	ratchet_fingerprint CHAR(32) NOT NULL,
	ratchet_key CHAR(32) NOT NULL
)
	", ()).unwrap();

	conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_peer_addr ON peers(salted_addr)", ()).unwrap();
	conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_ratchet_fingerprint ON ratchets(salted_fingerprint)", ()).unwrap();
	conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_invite_fingerprint ON invite_tokens(salted_fingerprint)", ()).unwrap();
	conn.execute("CREATE UNIQUE INDEX IF NOT EXISTS idx_invite_token ON invite_tokens(salted_token)", ()).unwrap();

	let token = [0u8; 8];
	let token_expiry = 111;
	let salted_token = [1u8; 16];
	let token_salted_fingerprint = [2u8; 16];
	let token_fingerprint = [3u8; 32];
	let token_key = [4u8; 32];

    let mut stmt = conn.prepare("INSERT INTO invite_tokens (salted_token,token,expiry_time,salted_fingerprint,ratchet_fingerprint,ratchet_key) VALUES(?1,?2,?3,?4,?5,?6)").unwrap();
	stmt.execute((salted_token, token, token_expiry, token_salted_fingerprint, token_fingerprint, token_key)).unwrap();
	drop(stmt);
	// Alice

	let mut is_invite_token = false;
	let peer_fingerprint = [3u8; 32];
	let peer_salted_fingerprint = [2u8; 16];
	let mut peer_key: [u8; 32] = [0u8; 32];
	{
		let mut stmt = conn.prepare("SELECT fingerprint, key, count FROM ratchets WHERE salted_fingerprint = ?1").unwrap();
		let mut rows = stmt.query([peer_salted_fingerprint]).unwrap();

		if let Some(row) = rows.next().unwrap() {
			println!("done");
			let fingerprint: [u8; 32] = row.get(0).unwrap();
			let peer_key: [u8; 32] = row.get(1).unwrap();
			let peer_count: u64 = row.get(2).unwrap();
			if peer_fingerprint == fingerprint {
				todo!();
			}
		} else {
			let mut stmt = conn.prepare("SELECT ratchet_fingerprint, ratchet_key, expiry_time FROM invite_tokens WHERE salted_fingerprint = ?1").unwrap();
			let mut rows = stmt.query([peer_salted_fingerprint]).unwrap();

			if let Some(row) = rows.next().unwrap() {
				let fingerprint: [u8; 32] = row.get(0).unwrap();
				peer_key = row.get(1).unwrap();
				let expiry_time: u64 = row.get(2).unwrap();
				//let peer_count: u64 = row.get(2).unwrap();
				if peer_fingerprint == fingerprint {//should be secure_eq
					is_invite_token = true;
				}
			}
		}
	}

	let peer_addr = [60u8; 16];
	let peer_salted_addr = [61u8; 16];
	// ZSSP handshake
	if is_invite_token {
		let tx = conn.transaction().unwrap();
		{
			let mut stmt = tx.prepare("DELETE FROM invite_tokens WHERE salted_fingerprint = ?1").unwrap();
			stmt.execute([peer_salted_fingerprint]).unwrap();

			let mut stmt = tx.prepare("
INSERT INTO peers (salted_addr, short_addr, status) VALUES(?1,?2,?3)
			").unwrap();
			stmt.execute((peer_salted_addr, peer_addr, b"frnd")).unwrap();

			let mut stmt = tx.prepare("
INSERT INTO ratchets (salted_fingerprint, fingerprint, key, count, peer) VALUES(?1,?2,?3,?4,?5)
			").unwrap();
			stmt.execute((peer_salted_fingerprint, peer_fingerprint, peer_key, 0, peer_salted_addr)).unwrap();
		}
		tx.commit().unwrap();
	}

	println!("done");
}
