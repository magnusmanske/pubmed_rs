//use rand::prelude::*;
use serde_json;

fn check_random_paper(client: &pubmed::Client) {
    let pmid = rand::random::<u64>() % 5e7 as u64;
    println!("Trying PMID https://eutils.ncbi.nlm.nih.gov/entrez/eutils/efetch.fcgi?db=pubmed&retmode=xml&id={}", pmid);
    let _articles = client.articles(&vec![pmid]);
}

// This is just used for testing
fn main() {
    let client = pubmed::Client::new();
    if true {
        loop {
            check_random_paper(&client);
        }
    } else {
        let articles = client.articles(&vec![10778477]).unwrap();
        if true {
            println!("{}", serde_json::to_string(&articles[0]).unwrap());
        }
    }
}
