fn main() {
    let client = pubmed::Client::new();
    /*
        let ids = client
            .work_ids_from_query(&"\"10.1016/j.bpj.2008.12.3951\"".to_string(), 1000)
            .unwrap();
        let works = client.works(&ids);
    */
    let works = client.works(&vec![19348744, 25081398]);
    println!("{:?}", works);
}
