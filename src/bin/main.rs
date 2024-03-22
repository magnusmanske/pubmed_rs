async fn check_random_paper(client: &pubmed::Client) {
    let pmid = rand::random::<u64>() % 5e7 as u64;
    println!("Trying PMID https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id={}", pmid);
    let _articles = client.articles(&[pmid]).await;
}

// This is just used for testing
#[tokio::main]
async fn main() {
    let client = pubmed::Client::new();
    if false {
        loop {
            check_random_paper(&client).await;
        }
    } else {
        let articles = client.articles(&[30947298]).await.unwrap();
        if true {
            println!("{}", serde_json::to_string(&articles[0]).unwrap());
        }
    }
}
