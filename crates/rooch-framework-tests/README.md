# Rooch framework tests


## How to build a bitcoin block tester genesis?

1. Download the events file `wget -c https://storage.googleapis.com/rooch_dev/ord_event_blk_858999.tar.zst`
2. Use unzstd and tar to decompressing the file to an event dir.
3. Run

```bash
cargo run -p rooch-framework-tests --  --btc-rpc-url http://localhost:9332 --btc-rpc-username your_username --btc-rpc-password your_pwd --ord-event-dir your_local_event_dir --blocks 790964 --blocks 855396
```