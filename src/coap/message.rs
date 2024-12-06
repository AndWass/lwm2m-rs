#[derive(Debug)]
pub struct Option {
    id: u16,
    value: Vec<u8>,
}

impl Option {
    pub fn new_string(id: u16, string: String) -> Option {
        Option {
            id,
            value: string.into_bytes(),
        }
    }
}

#[derive(Debug, Default)]
pub struct OptionBucket {
    options: Vec<Option>,
}

impl OptionBucket {
    pub const fn new() -> Self {
        Self {
            options: Vec::new(),
        }
    }
    pub fn encode_to(&self, out: &mut Vec<u8>) {
        let mut prev_option = 0;
        for option in &self.options {
            let delta = option.id - prev_option;
            if delta < 13 {
                out.push(delta as u8);
            }
            else if (delta - 13) < 256 {
                out.push(13);
                out.push((delta - 13) as u8);
            }
            else {
                out.push(14);
                let delta = delta - 269;
                out.extend(delta.to_be_bytes());
            }
            prev_option = option.id;
        }
    }

    pub fn push(&mut self, option: Option) {
        let insert_pos = {
            let mut insert_pos = self.options.len();
            for (i, x) in self.options.iter().enumerate() {
                if x.id > option.id {
                    insert_pos = i;
                    break;
                }
            }
            insert_pos
        };
        self.options.insert(insert_pos, option);
    }
}

#[derive(Debug, Copy, Clone)]
pub enum Type {
    Confirmable,
    NonConfirmable,
    Ack,
    Reset
}

pub struct Message {
    message_type: Type,
    code: u8,
    message_id: u16,

    token_length: u8,
    token: [u8; 8],

    options: Vec<Option>,
    payload: Vec<u8>,
}

impl Message {
    pub fn encode_to(&self, out: &mut Vec<u8>) {
        let mut first_byte = 0x02u8;
        match self.message_type {
            Type::Confirmable => {},
            Type::NonConfirmable => first_byte |= 0b00100,
            Type::Ack => first_byte |= 0b1000,
            Type::Reset => first_byte |= 0b1100,
        }

        let token_length = self.token_length & 0x0F;
        first_byte |= token_length << 4;
        out.clear();
        out.push(first_byte);
        out.push(self.code);
        out.extend(self.message_id.to_be_bytes());

        if token_length != 0 {
            out.extend_from_slice(&self.token[0..token_length as usize]);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn option_bucket() {
        let mut options = OptionBucket::new();
        options.push(Option::new_string(1, String::from("test")));
        options.push(Option::new_string(2, String::from("efg")));
        options.push(Option::new_string(1, String::from("abc")));

        assert_eq!(options.options[0].id, 1);
        assert_eq!(options.options[0].value, b"test");

        assert_eq!(options.options[1].id, 1);
        assert_eq!(options.options[1].value, b"abc");

        assert_eq!(options.options[2].id, 2);
        assert_eq!(options.options[2].value, b"efg");
    }
}