# S4 - Simpler Simple Storage Service

[![crates.io](https://meritbadge.herokuapp.com/s4)](https://crates.io/crates/s4)

:warning: **This create is still under heavy development and not yet production ready.**


## What is S4

S4 is attempt to provide a high-level API for S3. It is based on [Rusoto](https://www.rusoto.org/) and merely extents it's API.


## What is added that *Rusoto* itself doesn't provide

* simple way to create an `S3Client`
* download object to a file
* download object and [`Write`] it
* upload object from file
* [`Read`] object and upload it
* simple way to iterate through all objects or objects with a given prefix

## Implementation details

Most functionality is provided by the `S4` trait which is implemented for *Rusoto*'s `S3Client`.


[`Read`]: https://doc.rust-lang.org/nightly/std/io/trait.Read.html
[`Write`]: https://doc.rust-lang.org/nightly/std/io/trait.Write.html


## Running Tests

#. Start Minio

```
docker run -d --rm -p 9000:9000 --env "MINIO_ACCESS_KEY=ANTN35UAENTS5UIAEATD" \
--env "MINIO_SECRET_KEY=TtnuieannGt2rGuie2t8Tt7urarg5nauedRndrur" \
--env MINIO_DOMAIN=localhost minio/minio server /minio
```

#. Run tests

```
cargo test
```
