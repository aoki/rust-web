use reqwest::blocking::Client;

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
}
