# rusty_weed

A client implementation in rust for SeaweedFS.
Written with the help of reqwest as http client and serde for serialization/deserialization.

# Examples

## Upload bytes

```rust
let master = Master {
    host: MASTER_HOST.to_string(),
    port: Some(MASTER_PORT),
};

let options: AssignKeyOptions = Default::default();
let master_resp = master.assign_key(&Some(options)).await;

let fid: FID;
let volume: Volume;
match master_resp {
    Ok(x) => {
        println!("Address {}", x.location.url);
        volume = Volume::from_str(&x.location.url).unwrap();
        fid = x.fid;
    }
    _ => panic!("failed to assign key"),
}

let data = Bytes::from("Hello World!");
let resp = volume.upload_file_bytes(&fid, &data, &None).await;
```

# TODO

## Master endpoints

-   /vol/vacuum
-   /vol/grow
-   /col/delete
-   /cluster/status
-   /cluster/healthz
-   /dir/status
-   /vol/status

## Volume endpoints

-   Upload with `multipart/form-data`
-   /status

## Filer endpoints

-   all
