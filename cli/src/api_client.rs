use reqwest::blocking::Client;
use std::io;

pub struct ApiClient {
    pub server: String,
    pub client: Client,
}

impl ApiClient {
    pub fn psot_logs(&self, req: &api::logs::post::Request) -> reqwest::Result<()> {
        self.client
            .post(&format!("http://{}/logs", &self.server))
            .json(req)
            .send()
            .map(|_| ())
    }

    pub fn get_logs(&self) -> reqwest::Result<api::logs::get::Response> {
        self.client
            .get(&format!("http://{}/logs", &self.server))
            .send()?
            .json()
    }

    pub fn get_csv<W: io::Write>(&self, w: &mut W) -> reqwest::Result<u64> {
        self.client
            .get(&format!("http://{}/csv", &self.server))
            .send()?
            .copy_to(w)
    }
}
