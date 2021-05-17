use itertools::Itertools;

pub struct Model {
    composed_email_content: String,
    selected_channel_name: String,
    emails: Vec<String>,
}

fn email_channel(email: &str) -> String {
    match email.find(':') {
        Some(i) => email[0..i].to_owned(),
        _ => "misc".to_owned(),
    }
}

impl Model {
    pub fn default() -> Model {
        Model {
            composed_email_content: "".to_owned(),
            selected_channel_name: "+".to_string(),
            emails: vec![],
        }
    }

    pub fn replace_emails(&mut self, emails: Vec<String>) {
        self.emails = emails;
    }

    pub fn composed(&self) -> String {
        self.composed_email_content.clone()
    }

    pub fn composed_backspace(&mut self) {
        let len = self.composed_email_content.len();
        if len > 0 {
            self.composed_email_content.remove(len - 1);
        }
    }

    pub fn composed_push(&mut self, to_push: char) {
        self.composed_email_content.push(to_push);
    }

    pub fn composed_clear(&mut self) {
        self.composed_email_content.clear();
    }

    pub fn select_channel(&mut self, channel: &str) {
        self.selected_channel_name = channel.to_string();
    }

    pub fn selected_channel_idx(&self) -> usize {
        let channels = self.channels();
        channels
            .iter()
            .find_position(|channel| channel.eq(&&self.selected_channel_name))
            .map(|it| it.0)
            .unwrap_or(channels.len())
    }

    pub fn selected_channel_name(&self) -> String {
        self.selected_channel_name.clone()
    }

    pub fn emails_for_selected_channel(&self) -> Vec<String> {
        return self
            .emails
            .iter()
            .filter(|email| email_channel(email) == self.selected_channel_name.as_str())
            .map(|s| s.to_owned())
            .collect();
    }

    pub fn channels(&self) -> Vec<String> {
        let mut channels: Vec<String> = self
            .emails
            .clone()
            .into_iter()
            .map(|email| email_channel(email.as_str()))
            .unique()
            .sorted();
        channels.push("+".to_owned());
        channels
    }

    pub fn dec_channel(&mut self) {
        let new_idx = self.selected_channel_idx().saturating_sub(1).max(0);
        self.selected_channel_name = self.channels()[new_idx].to_owned();
    }

    pub fn inc_channel(&mut self) {
        let new_idx = self
            .selected_channel_idx()
            .checked_add(1)
            .unwrap_or(0)
            .min(self.channels().len() - 1);

        self.selected_channel_name = self.channels()[new_idx].to_owned();
    }
}
