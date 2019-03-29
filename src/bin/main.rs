use serde_json;

fn main() {
    let client = pubmed::Client::new();
    /*
        let ids = client
            .work_ids_from_query(&"\"10.1016/j.bpj.2008.12.3951\"".to_string(), 1000)
            .unwrap();
        let works = client.works(&ids);
    */
    let articles = client.articles(&vec![19348744]).unwrap(); // 22722859,19348744,25081398
    println!("{}", serde_json::to_string(&articles[0]).unwrap());
}
