Key queries that must be supported:
* Map a ratchet fingerprint to a ratchet key and ratchet count
* Map a peer's short address to an identity, and up to two sets of ratchet states (a ratchet fingerprint, key and count)
* Track whether peers are friends or roots, with corresponding metadata.
* Atomically add and remove ratchets assigned to a peer.

Invite token queries:
* Add an invite token with a corresponding ratchet state
* Lookup an invite token by hash.
* Map a ratchet fingerprint to its corresponding invite token (if it has one)
* Atomically add a peer as a friend and remove an invite token and its ratchet state

Security properties:
* All search indices must be salted and hashed to prevent timing attacks. (Salting provides anonimity, hashing provides confidentiality.)
* We want to encrypt sensitive fields (ratchet states, invite tokens, etc.) so users are less likely to compromise themselves when making support requests.
* Invite tokens should be removed after an arbitrary time-to-live.


SQLite Schema:
```sql
CREATE TABLE IF NOT EXISTS peers (
	salted_addr CHAR(16) PRIMARY KEY,
	short_addr CHAR(16) NOT NULL,
	identity TEXT,
	ip TEXT,
	status CHAR(4)
);
CREATE TABLE IF NOT EXISTS ratchets (
	salted_fingerprint CHAR(16) PRIMARY KEY,
	fingerprint CHAR(32),
	key CHAR(32) NOT NULL,
	count BIGINT NOT NULL,

	peer CHAR(16) NOT NULL,
	FOREIGN KEY (peer) REFERENCES peers(salted_addr)
		ON DELETE CASCADE
);
CREATE TABLE IF NOT EXISTS invite_tokens (
	salted_token CHAR(16) PRIMARY KEY,
	token VARCHAR(32) NOT NULL,
    expiry_time BIGINT NOT NULL,

	salted_fingerprint CHAR(16) UNIQUE NOT NULL,
	ratchet_fingerprint CHAR(32) NOT NULL,
	ratchet_key CHAR(32) NOT NULL
);
CREATE UNIQUE INDEX IF NOT EXISTS idx_peer_addr ON peers(salted_addr);
CREATE UNIQUE INDEX IF NOT EXISTS idx_ratchet_fingerprint ON ratchets(salted_fingerprint);
CREATE UNIQUE INDEX IF NOT EXISTS idx_invite_fingerprint ON invite_tokens(salted_fingerprint);
CREATE UNIQUE INDEX IF NOT EXISTS idx_invite_token ON invite_tokens(salted_token);
```
