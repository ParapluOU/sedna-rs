use sedna_rs::{Result, SednaServer};

fn main() -> Result<()> {
    println!("Starting embedded Sedna XML Database...");

    // Start an embedded Sedna server
    let server = SednaServer::new()?;
    println!("Server started on port {}", server.port());

    // Connect to the default test database
    let mut client = server.connect("testdb", "SYSTEM", "MANAGER")?;
    println!("Connected to database");

    // Create a collection
    client.begin_transaction()?;
    client.execute("CREATE COLLECTION 'docs'")?;
    client.commit_transaction()?;
    println!("Collection created");

    // Load XML document using SEloadData (programmatic LOAD)
    client.begin_transaction()?;
    let xml_data = "<book><title>The Rust Programming Language</title><author>Steve Klabnik</author><author>Carol Nichols</author><year>2018</year></book>";
    client.load_xml_data(xml_data, "test", Some("docs"))?;
    client.commit_transaction()?;
    println!("Document loaded");

    // Query the document
    println!("\nQuerying for book title...");
    client.begin_transaction()?;
    let mut result = client.execute("doc('test', 'docs')//title")?;
    while let Some(item) = result.next()? {
        println!("Result: {}", item);
    }
    client.commit_transaction()?;

    // Query for all authors
    println!("\nQuerying for all authors...");
    client.begin_transaction()?;
    let mut result = client.execute("doc('test', 'docs')//author")?;
    while let Some(item) = result.next()? {
        println!("Author: {}", item);
    }
    client.commit_transaction()?;

    // Update query
    println!("\nUpdating the year...");
    client.begin_transaction()?;
    client.execute("UPDATE replace $x in doc('test', 'docs')//year with <year>2023</year>")?;
    client.commit_transaction()?;

    // Verify the update
    println!("\nVerifying update...");
    client.begin_transaction()?;
    let mut result = client.execute("doc('test', 'docs')//year")?;
    while let Some(item) = result.next()? {
        println!("Year: {}", item);
    }
    client.commit_transaction()?;

    println!("\nQuickstart completed successfully!");

    Ok(())
}
