mod common;
use crate::common::*;

use fallible_iterator::FallibleIterator;
use rusoto_s3::GetObjectOutput;
use s4::error::S4Result;
use s4::S4;
use tokio::io::{AsyncReadExt};

#[tokio::test]
async fn iter_objects() {
    let (client, bucket) = create_test_bucket().await;

    for i in (0..2003).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, vec![]).await;
    }

    let mut iter = client.iter_objects(&bucket).await;
    for i in (0..2003).map(|i| format!("{:04}", i)) {
        let object = iter.next().unwrap().unwrap();
        assert_eq!(object.key.unwrap(), i);
    }
    assert!(iter.next().unwrap().is_none());
}

#[tokio::test]
async fn iter_objects_with_prefix() {
    let (client, bucket) = create_test_bucket().await;

    for i in (0..1005).map(|i| format!("a/{:04}", i)) {
        put_object(&client, &bucket, &i, vec![]).await;
    }
    put_object(&client, &bucket, "b/1234", vec![]).await;

    let mut iter = client.iter_objects_with_prefix(&bucket, "a/").await;
    for i in (0..1005).map(|i| format!("a/{:04}", i)) {
        let object = iter.next().unwrap().unwrap();
        assert_eq!(object.key.unwrap(), i);
    }
    assert!(iter.next().unwrap().is_none());
}

#[tokio::test]
async fn iter_objects_nth() {
    let (client, bucket) = create_test_bucket().await;

    for i in (1..2081).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, vec![]).await;
    }

    let mut iter = client.iter_objects(&bucket).await;
    let obj = iter.nth(0).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "0001");
    let obj = iter.nth(2).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "0004");
    let obj = iter.nth(1999).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "2004");
    let obj = iter.nth(75).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "2080");
    assert!(iter.nth(0).unwrap().is_none());
    assert!(iter.nth(3).unwrap().is_none());

    let mut iter = client.iter_objects(&bucket).await;
    let obj = iter.nth(1000).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "1001");
    let obj = iter.nth(997).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "1999");
    let obj = iter.nth(0).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "2000");
    let obj = iter.nth(0).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "2001");

    let mut iter = client.iter_objects(&bucket).await;
    let obj = iter.nth(2030).unwrap().unwrap();
    assert_eq!(obj.key.unwrap(), "2031");
}

#[tokio::test]
async fn iter_objects_count() {
    let (client, bucket) = create_test_bucket().await;

    assert_eq!(client.iter_objects(&bucket).await.count().unwrap(), 0);

    for i in (0..2122).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, vec![]).await;
    }

    assert_eq!(client.iter_objects(&bucket).await.count().unwrap(), 2122);

    let mut iter = client.iter_objects(&bucket).await;
    iter.nth(1199).unwrap().unwrap();
    assert_eq!(iter.count().unwrap(), 922);

    let mut iter = client.iter_objects(&bucket).await;
    iter.nth(2120).unwrap().unwrap();
    assert_eq!(iter.count().unwrap(), 1);

    let mut iter = client.iter_objects(&bucket).await;
    iter.nth(2121).unwrap().unwrap();
    assert_eq!(iter.count().unwrap(), 0);

    let mut iter = client.iter_objects(&bucket).await;
    assert!(iter.nth(2122).unwrap().is_none());
    assert_eq!(iter.count().unwrap(), 0);
}

#[tokio::test]
async fn iter_objects_last() {
    let (client, bucket) = create_test_bucket().await;

    assert!(client.iter_objects(&bucket).await.last().unwrap().is_none());

    for i in (1..1000).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, vec![]).await;
    }

    assert_eq!(
        client
            .iter_objects(&bucket)
            .await
            .last()
            .unwrap()
            .unwrap()
            .key
            .unwrap(),
        "0999"
    );
    put_object(&client, &bucket, "1000", vec![]).await;
    assert_eq!(
        client
            .iter_objects(&bucket)
            .await
            .last()
            .unwrap()
            .unwrap()
            .key
            .unwrap(),
        "1000"
    );
    put_object(&client, &bucket, "1001", vec![]).await;
    assert_eq!(
        client
            .iter_objects(&bucket)
            .await
            .last()
            .unwrap()
            .unwrap()
            .key
            .unwrap(),
        "1001"
    );
}

#[tokio::test]
async fn iter_get_objects() {
    let (client, bucket) = create_test_bucket().await;

    for i in (1..1004).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, i.clone().into_bytes()).await;
    }

    let mut iter = client.iter_get_objects(&bucket).await;
    for i in (1..1004).map(|i| format!("{:04}", i)) {
        let (key, obj) = iter.next().unwrap().unwrap();
        let mut body = Vec::new();
        obj.body
            .unwrap()
            .into_async_read()
            .read_to_end(&mut body)
            .await
            .map(|_| ())
            .unwrap();
        assert_eq!(key, i);
        assert_eq!(body, i.as_bytes());
    }
    assert!(iter.next().unwrap().is_none());
}

#[tokio::test]
async fn iter_get_objects_nth() {
    let (client, bucket) = create_test_bucket().await;

    for i in (1..1003).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, i.clone().into_bytes()).await;
    }

    let mut iter = client.iter_get_objects(&bucket).await;
    assert_key_and_body(iter.nth(0), "0001").await;
    assert_key_and_body(iter.nth(997), "0999").await;
    assert_key_and_body(iter.nth(0), "1000").await;
    assert_key_and_body(iter.nth(0), "1001").await;
    assert_key_and_body(iter.nth(0), "1002").await;
    assert!(iter.nth(0).unwrap().is_none());
}

#[tokio::test]
async fn iter_get_objects_with_prefix_count() {
    let (client, bucket) = create_test_bucket().await;

    put_object(&client, &bucket, "a/0020", vec![]).await;
    put_object(&client, &bucket, "c/0030", vec![]).await;
    assert_eq!(
        client
            .iter_get_objects_with_prefix(&bucket, "b/")
            .await
            .count()
            .unwrap(),
        0
    );

    for i in (0..533).map(|i| format!("b/{:04}", i)) {
        put_object(&client, &bucket, &i, i.clone().into_bytes()).await;
    }

    assert_eq!(
        client
            .iter_get_objects_with_prefix(&bucket, "b/")
            .await
            .count()
            .unwrap(),
        533
    );
}

#[tokio::test]
async fn iter_get_objects_last() {
    let (client, bucket) = create_test_bucket().await;

    assert!(client
        .iter_get_objects(&bucket)
        .await
        .last()
        .unwrap()
        .is_none());

    for i in (1..1002).map(|i| format!("{:04}", i)) {
        put_object(&client, &bucket, &i, i.clone().into_bytes()).await;
    }

    assert_key_and_body(client.iter_get_objects(&bucket).await.last(), "1001").await;
}

async fn assert_key_and_body(output: S4Result<Option<(String, GetObjectOutput)>>, expected: &str) {
    let (key, object) = output.unwrap().unwrap();

    let mut body = Vec::new();
    object
        .body
        .unwrap()
        .into_async_read()
        .read_to_end(&mut body)
        .await
        .unwrap();

    assert_eq!(key, expected);
    assert_eq!(body, expected.as_bytes());
}
