use s3s::xml;

use std::fmt;

use rust_utils::default::default;

fn deserialize_content<T>(input: &[u8]) -> xml::DeResult<T>
where
    T: for<'xml> xml::DeserializeContent<'xml>,
{
    let mut d = xml::Deserializer::new(input);
    let ans = T::deserialize_content(&mut d)?;
    d.expect_eof()?;
    Ok(ans)
}

fn deserialize<T>(input: &[u8]) -> xml::DeResult<T>
where
    T: for<'xml> xml::Deserialize<'xml>,
{
    let mut d = xml::Deserializer::new(input);
    let ans = T::deserialize(&mut d)?;
    d.expect_eof()?;
    Ok(ans)
}

fn serialize_content<T: xml::SerializeContent>(val: &T) -> xml::SerResult<String> {
    let mut buf = Vec::with_capacity(256);
    {
        let mut ser = xml::Serializer::new(&mut buf);
        val.serialize_content(&mut ser)?;
    }
    Ok(String::from_utf8(buf).unwrap())
}

fn serialize<T: xml::Serialize>(val: &T) -> xml::SerResult<String> {
    let mut buf = Vec::with_capacity(256);
    {
        let mut ser = xml::Serializer::new(&mut buf);
        val.serialize(&mut ser)?;
    }
    Ok(String::from_utf8(buf).unwrap())
}

fn test_serde<T>(val: &T)
where
    T: for<'xml> xml::Deserialize<'xml>,
    T: xml::Serialize,
    T: fmt::Debug + PartialEq,
{
    let xml = serialize(val).unwrap();
    let ans = deserialize::<T>(xml.as_bytes()).unwrap();
    assert_eq!(*val, ans);
}

fn test_serde_content<T>(val: &T)
where
    T: for<'xml> xml::DeserializeContent<'xml>,
    T: xml::SerializeContent,
    T: fmt::Debug + PartialEq,
{
    let xml = serialize_content(val).unwrap();
    let ans = deserialize_content::<T>(xml.as_bytes()).unwrap();
    assert_eq!(*val, ans);
}

/// See <https://github.com/Nugine/s3s/issues/2>
#[test]
fn completed_multipart_upload() {
    let input = r#"
        <CompleteMultipartUpload>
            <Part>
                <PartNumber>1</PartNumber>
                <ETag>"a54357aff0632cce46d942af68356b38"</ETag>
            </Part>
            <Part>
                <PartNumber>2</PartNumber>
                <ETag>"0c78aef83f66abc1fa1e8477f296d394"</ETag>
            </Part>
            <Part>
                <PartNumber>3</PartNumber>
                <ETag>"acbd18db4cc2f85cedef654fccc4a4d8"</ETag>
            </Part>
        </CompleteMultipartUpload>
    "#;

    let ans = deserialize::<s3s::dto::CompletedMultipartUpload>(input.as_bytes()).unwrap();

    let parts = ans.parts.as_deref().unwrap();
    assert_eq!(parts.len(), 3);

    assert_eq!(parts[0].part_number, Some(1));
    assert_eq!(parts[0].e_tag.as_deref(), Some("\"a54357aff0632cce46d942af68356b38\""));

    assert_eq!(parts[1].part_number, Some(2));
    assert_eq!(parts[1].e_tag.as_deref(), Some("\"0c78aef83f66abc1fa1e8477f296d394\""));

    assert_eq!(parts[2].part_number, Some(3));
    assert_eq!(parts[2].e_tag.as_deref(), Some("\"acbd18db4cc2f85cedef654fccc4a4d8\""));

    test_serde(&ans);
}

#[test]
fn select_object_content_request() {
    {
        let input = r#"
        <SelectObjectContentRequest xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <Expression>select * from s3object</Expression>
            <ExpressionType>SQL</ExpressionType>
            <InputSerialization>
                <CompressionType>NONE</CompressionType>
                <CSV>
                    <AllowQuotedRecordDelimiter>FALSE</AllowQuotedRecordDelimiter>
                    <Comments>#</Comments>
                    <FieldDelimiter>,</FieldDelimiter>
                    <FileHeaderInfo>NONE</FileHeaderInfo>
                    <QuoteCharacter>""</QuoteCharacter>
                    <QuoteEscapeCharacter>"</QuoteEscapeCharacter>
                    <RecordDelimiter>\n</RecordDelimiter>
                </CSV>
            </InputSerialization>
            <OutputSerialization>
                <JSON>
                    <RecordDelimiter>\n</RecordDelimiter>
                </JSON>
            </OutputSerialization>
        </SelectObjectContentRequest>
    "#;

        let ans = deserialize::<s3s::dto::SelectObjectContentRequest>(input.as_bytes()).unwrap();

        {
            let csv = ans.input_serialization.csv.as_ref().unwrap();
            assert_eq!(csv.allow_quoted_record_delimiter, Some(false));
        }

        test_serde(&ans);
    }

    {
        let input = r#"
    <SelectObjectContentRequest xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
        <Expression>select * from s3object</Expression>
        <ExpressionType>SQL</ExpressionType>
        <InputSerialization>
            <CSV />
        </InputSerialization>
        <OutputSerialization>
            <CSV />
        </OutputSerialization>
        <RequestProgress>
            <Enabled>true</Enabled>
        </RequestProgress>
    </SelectObjectContentRequest>
"#;

        let ans = deserialize::<s3s::dto::SelectObjectContentRequest>(input.as_bytes()).unwrap();

        assert!(ans.input_serialization.csv.is_some());
        assert!(ans.output_serialization.csv.is_some());

        test_serde(&ans);
    }
}

#[test]
fn tagging() {
    let input = r#"
        <Tagging xmlns="http://s3.amazonaws.com/doc/2006-03-01/">
            <TagSet>
                <Tag>
                    <Key>Key4</Key>
                    <Value>Value4</Value>
                </Tag>
            </TagSet>
        </Tagging>
    "#;

    let ans = deserialize::<s3s::dto::Tagging>(input.as_bytes()).unwrap();

    assert_eq!(ans.tag_set.len(), 1);
    let tag = &ans.tag_set[0];
    assert_eq!(tag.key, "Key4");
    assert_eq!(tag.value, "Value4");

    test_serde(&ans);
}

#[test]
fn lifecycle_expiration() {
    let val = s3s::dto::LifecycleExpiration {
        date: None,
        days: Some(365),
        expired_object_delete_marker: None,
    };

    let ans = serialize_content(&val).unwrap();
    let expected = "<Days>365</Days>";

    assert_eq!(ans, expected);

    test_serde_content(&val);
}

#[test]
fn get_bucket_location_output() {
    {
        let us_west_2 = s3s::dto::BucketLocationConstraint::from_static(s3s::dto::BucketLocationConstraint::US_WEST_2);
        let val = s3s::dto::GetBucketLocationOutput {
            location_constraint: Some(us_west_2),
        };

        let ans = serialize(&val).unwrap();
        let expected = "<LocationConstraint>us-west-2</LocationConstraint>";

        assert_eq!(ans, expected);

        test_serde(&val);
    }
    {
        let val = s3s::dto::GetBucketLocationOutput {
            location_constraint: None,
        };

        let ans = serialize(&val).unwrap();
        let expected = "<LocationConstraint></LocationConstraint>";

        assert_eq!(ans, expected);

        test_serde(&val);
    }
}

#[test]
fn get_bucket_notification_configuration_output() {
    let xml = "<NotificationConfiguration></NotificationConfiguration>";

    let val = deserialize::<s3s::dto::NotificationConfiguration>(xml.as_bytes()).unwrap();
    assert_eq!(val, default());
    test_serde(&val);

    let val = deserialize::<s3s::dto::GetBucketNotificationConfigurationOutput>(xml.as_bytes()).unwrap();
    assert_eq!(val, default());
    test_serde(&val);
}