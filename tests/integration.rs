use sedna_rs::{Result, SednaServer};

#[test]
fn test_server_startup() -> Result<()> {
    let server = SednaServer::new()?;
    assert!(server.port() > 0);
    Ok(())
}

#[test]
fn test_client_connection() -> Result<()> {
    let server = SednaServer::new()?;
    let _client = server.connect("testdb", "SYSTEM", "MANAGER")?;
    Ok(())
}

#[test]
fn test_simple_query() -> Result<()> {
    let server = SednaServer::new()?;
    let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;

    // Begin transaction
    client.begin_transaction()?;

    // Load a document
    client.execute(
        "CREATE DOCUMENT 'test1' IN COLLECTION 'col1' <root><item>Hello World</item></root>",
    )?;

    // Commit
    client.commit_transaction()?;

    // Query the document
    let mut result = client.execute("doc('test1')//item")?;

    let items = result.collect_all()?;
    assert_eq!(items.len(), 1);
    assert!(items[0].contains("Hello World"));

    Ok(())
}

#[test]
fn test_transaction_rollback() -> Result<()> {
    let server = SednaServer::new()?;
    let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;

    // Begin and commit a transaction to create a document
    client.begin_transaction()?;
    client.execute("CREATE DOCUMENT 'test2' IN COLLECTION 'col2' <root><value>1</value></root>")?;
    client.commit_transaction()?;

    // Begin a new transaction and update
    client.begin_transaction()?;
    client.execute("UPDATE replace $x in doc('test2')//value with <value>2</value>")?;

    // Rollback the transaction
    client.rollback_transaction()?;

    // Query should show original value
    let mut result = client.execute("doc('test2')//value")?;
    let items = result.collect_all()?;
    assert!(items[0].contains("<value>1</value>"));

    Ok(())
}

#[test]
fn test_multiple_results() -> Result<()> {
    let server = SednaServer::new()?;
    let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;

    client.begin_transaction()?;
    client.execute(
        "CREATE DOCUMENT 'test3' IN COLLECTION 'col3' <books><book><title>Book 1</title></book><book><title>Book 2</title></book><book><title>Book 3</title></book></books>",
    )?;
    client.commit_transaction()?;

    let mut result = client.execute("doc('test3')//title")?;
    let items = result.collect_all()?;

    assert_eq!(items.len(), 3);
    assert!(items[0].contains("Book 1"));
    assert!(items[1].contains("Book 2"));
    assert!(items[2].contains("Book 3"));

    Ok(())
}

#[test]
fn test_empty_result() -> Result<()> {
    let server = SednaServer::new()?;
    let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;

    let mut result = client.execute("doc('nonexistent')//item")?;
    let items = result.collect_all()?;

    assert_eq!(items.len(), 0);

    Ok(())
}

#[test]
fn test_multiple_servers() -> Result<()> {
    let server1 = SednaServer::with_port(5050)?;
    let server2 = SednaServer::with_port(5060)?;

    assert_ne!(server1.port(), server2.port());

    let _client1 = server1.connect("testdb", "SYSTEM", "MANAGER")?;
    let _client2 = server2.connect("testdb", "SYSTEM", "MANAGER")?;

    Ok(())
}
