use serde::{Deserialize, Serialize};

#[derive(Debug, Deserialize, Serialize)]
pub enum Identifier {
    Index(usize),
    NumberOcc(usize),
}
impl Identifier {
    pub fn start(&self, link: &str) -> usize {
        match self {
            Identifier::Index(i) => *i,
            Identifier::NumberOcc(i) => {
                let mut result = 0;
                let mut count = 0;
                let chars = link.chars();
                let mut in_number = false;
                chars.into_iter().enumerate().for_each(|(index, c)| {
                    if c.is_ascii_digit() && !in_number {
                        in_number = true;
                        if *i == count {
                            result = index;
                            return;
                        } else {
                            count += 1;
                        }
                    }
                    if in_number && !c.is_ascii_digit() {
                        in_number = false;
                    }
                });
                result
            }
        }
    }
    pub fn end(&self, link: &str) -> usize {
        let start = self.start(link);
        start
            + link[start..]
                .chars()
                .take_while(|c| c.is_ascii_digit())
                .count()
    }
}
