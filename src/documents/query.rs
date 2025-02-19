use crate::{errors::extract_google_api_error_async, FirebaseAuthBearerAsync};

use super::*;
use std::vec::IntoIter;

///
/// Queries the database for specific documents, for example all documents in a collection of 'type' == "car".
///
/// Example:
/// ```no_run
/// # use serde::{Serialize, Deserialize};
/// #[derive(Debug, Serialize, Deserialize)]
/// struct DemoDTO { a_string: String, an_int: u32, }
///
/// use firestore_db_and_auth::{documents, dto};
/// # use firestore_db_and_auth::{credentials::Credentials, ServiceSession, errors::Result};
/// # use firestore_db_and_auth::credentials::doctest_credentials;
/// # let session = ServiceSession::new(doctest_credentials())?;
///
/// let values: documents::Query = documents::query(&session, "tests", "Sam Weiss".into(), dto::FieldOperator::EQUAL, "id")?;
/// for metadata in values {
///     println!("id: {}, created: {}, updated: {}", &metadata.name, metadata.create_time.as_ref().unwrap(), metadata.update_time.as_ref().unwrap());
///     // Fetch the actual document
///     // The data is wrapped in a Result<> because fetching new data could have failed
///     let doc : DemoDTO = documents::read_by_name(&session, &metadata.name)?;
///     println!("{:?}", doc);
/// }
/// # Ok::<(), firestore_db_and_auth::errors::FirebaseError>(())
/// ```
///
/// ## Arguments
/// * 'auth' The authentication token
/// * 'collectionid' The collection id; "my_collection" or "a/nested/collection"
/// * 'value' The query / filter value. For example "car".
/// * 'operator' The query operator. For example "EQUAL".
/// * 'field' The query / filter field. For example "type".
pub fn query(
    auth: &impl FirebaseAuthBearer,
    collection_id: &str,
    value: serde_json::Value,
    operator: dto::FieldOperator,
    field: &str,
) -> Result<Query> {
    let url = firebase_url_query(auth.project_id());
    let value = crate::firebase_rest_to_rust::serde_value_to_firebase_value(&value);

    let query_request = dto::RunQueryRequest {
        structured_query: Some(dto::StructuredQuery {
            select: Some(dto::Projection { fields: None }),
            where_: Some(dto::Filter {
                field_filter: Some(dto::FieldFilter {
                    value,
                    op: operator,
                    field: dto::FieldReference {
                        field_path: field.to_owned(),
                    },
                }),
                ..Default::default()
            }),
            from: Some(vec![dto::CollectionSelector {
                collection_id: Some(collection_id.to_owned()),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resp = auth
        .client()
        .post(url)
        .bearer_auth(auth.access_token().to_owned())
        .json(&query_request)
        .send()?;

    let resp = extract_google_api_error(resp, || collection_id.to_owned())?;

    let json: Option<Vec<dto::RunQueryResponse>> = resp.json()?;

    Ok(Query(json.unwrap_or_default().into_iter()))
}

///
/// Queries the database for specific documents, for example all documents in a collection of 'type' == "car".
///
/// Example:
/// ```no_run
/// # use serde::{Serialize, Deserialize};
/// #[derive(Debug, Serialize, Deserialize)]
/// struct DemoDTO { a_string: String, an_int: u32, }
///
/// use firestore_db_and_auth::{documents, dto};
/// # use firestore_db_and_auth::{credentials::Credentials, ServiceSession, errors::Result};
/// # use firestore_db_and_auth::credentials::doctest_credentials;
/// # let session = ServiceSession::new(doctest_credentials())?;
///
/// let values: documents::Query = documents::query(&session, "tests", "Sam Weiss".into(), dto::FieldOperator::EQUAL, "id")?;
/// for metadata in values {
///     println!("id: {}, created: {}, updated: {}", &metadata.name, metadata.create_time.as_ref().unwrap(), metadata.update_time.as_ref().unwrap());
///     // Fetch the actual document
///     // The data is wrapped in a Result<> because fetching new data could have failed
///     let doc : DemoDTO = documents::read_by_name(&session, &metadata.name)?;
///     println!("{:?}", doc);
/// }
/// # Ok::<(), firestore_db_and_auth::errors::FirebaseError>(())
/// ```
///
/// ## Arguments
/// * 'auth' The authentication token
/// * 'collectionid' The collection id; "my_collection" or "a/nested/collection"
/// * 'value' The query / filter value. For example "car".
/// * 'operator' The query operator. For example "EQUAL".
/// * 'field' The query / filter field. For example "type".
/// THIS IS A NON-BLOCKING OPERATION
pub async fn query_async(
    auth: &mut impl FirebaseAuthBearerAsync,
    collection_id: &str,
    value: serde_json::Value,
    operator: dto::FieldOperator,
    field: &str,
) -> Result<Query> {
    let url = firebase_url_query(auth.project_id());
    let value = crate::firebase_rest_to_rust::serde_value_to_firebase_value(&value);

    let query_request = dto::RunQueryRequest {
        structured_query: Some(dto::StructuredQuery {
            select: Some(dto::Projection { fields: None }),
            where_: Some(dto::Filter {
                field_filter: Some(dto::FieldFilter {
                    value,
                    op: operator,
                    field: dto::FieldReference {
                        field_path: field.to_owned(),
                    },
                }),
                ..Default::default()
            }),
            from: Some(vec![dto::CollectionSelector {
                collection_id: Some(collection_id.to_owned()),
                ..Default::default()
            }]),
            ..Default::default()
        }),
        ..Default::default()
    };

    let resp = auth
        .client_async()
        .post(&url)
        .bearer_auth(auth.access_token().await.to_string())
        .json(&query_request)
        .send()
        .await?;

    let resp = extract_google_api_error_async(resp, || collection_id.to_owned()).await?;

    let json: Option<Vec<dto::RunQueryResponse>> = resp.json().await?;

    Ok(Query(json.unwrap_or_default().into_iter()))
}

/// This type is returned as a result by [`query()`].
/// Use it as an iterator. The query API returns a list of document references, not the documents itself.
///
/// If you just need the meta data like the document name or update time, you are already settled.
/// To fetch the document itself, use [`read_by_name`].
///
/// Please note that this API acts as an iterator of same-like documents.
/// This type is not suitable if you want to list documents of different types.
pub struct Query(IntoIter<dto::RunQueryResponse>);

impl Iterator for Query {
    type Item = dto::Document;

    // Skip empty entries
    fn next(&mut self) -> Option<Self::Item> {
        for r in self.0.by_ref() {
            if let Some(document) = r.document {
                return Some(document);
            }
        }
        None
    }
}
