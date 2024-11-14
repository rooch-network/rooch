# Backend

Implementations of DA backend.

## Open-DA

> - fs: local/remote file system
>- avail: Avail project DA
>- celestia: Celestia DA

## New Backend

For new added backend:

If it could satisfy open-da config, it should be added to `open-da` folder as a module. If not, it should be added to
`backend` folder directly.

## Backend Implementations & Verification

| Name     | Description                                | Category | Implementation               | Local | Testnet | Mainnet |
|----------|--------------------------------------------|----------|------------------------------|-------|---------|---------|
| fs       | file I/O based on local/remote file system | open-da  | [fs](open-da/fs)             | ✅     | ✅       | ✅       |
| avail    | Avail project DA                           | open-da  | [avail](open-da/avail)       | 🔲    | 🔲      | 🔲      |
| celestia | Celestia DA                                | open-da  | [celestia](open-da/celestia) | 🔲    | 🔲      | 🔲      |
| gcs      | file I/O based on Google Cloud Storage     | open-da  | [gcs](open-da/fs)            | ✅     | ✅       | ✅       |

- [x] ✅ done
- [ ] 🔲 unfinished
- [ ] ❌ has issues